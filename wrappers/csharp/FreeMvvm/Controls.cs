using System;
using System.Runtime.InteropServices;
using System.Text;

namespace FreeMvvm;

public enum UiControlOrientation : uint
{
    Vertical = 1,
    Horizontal = 2,
}

public enum UiChildAlignment : uint
{
    Start = 1,
    Center = 2,
    End = 3,
    Stretch = 4,
}

public abstract class UiWidget : IDisposable
{
    private static readonly UTF8Encoding StrictUtf8 = new(false, true);
    private readonly TokenOwner _owner;
    private readonly TokenOwner _runtimeAnchor;
    private readonly CallbackHandlerState _callbackHandlers;

    internal UiWidget(TokenOwner owner, TokenOwner runtimeAnchor, CallbackHandlerState callbackHandlers)
    {
        _owner = owner;
        _runtimeAnchor = runtimeAnchor;
        _callbackHandlers = callbackHandlers;
    }

    internal TokenOwner Owner => _owner;

    internal CallbackHandlerState CallbackHandlers => _callbackHandlers;

    internal INativeApi Api => _owner.Api;

    public UiResult<UiProperty<bool>> TryGetIsEnabledProperty() => GetProperty<bool>(
        static (INativeApi api, ulong widget, out ulong property) => api.WidgetIsEnabledProperty(widget, out property),
        owner => UiProperty<bool>.FromOwned(owner, _callbackHandlers));

    public UiResult<UiProperty<string>> TryGetAccessibleNameProperty() => GetProperty<string>(
        static (INativeApi api, ulong widget, out ulong property) => api.WidgetAccessibleNameProperty(widget, out property),
        owner => UiProperty<string>.FromOwned(owner, _callbackHandlers));

    public UiResult<bool> TryRequestFocus()
    {
        UiResult<CallLease> leaseResult = _owner.AcquireLease();
        if (leaseResult.IsFailure)
        {
            return UiResult<bool>.Failure(leaseResult.Error!);
        }

        using CallLease lease = leaseResult.Value;
        var preparedError = new PreparedError(Api);
        try
        {
            uint status = Api.WidgetRequestFocus(lease.Token, out ulong error);
            return status == UiStatus.Ok
                ? UiResult<bool>.Success(true)
                : UiResult<bool>.Failure(preparedError.Capture(status, error));
        }
        catch
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    public UiResult<UiCloseDisposition> TryClose() => _owner.TryClose();

    public void Dispose()
    {
        _owner.Dispose();
        _runtimeAnchor.Dispose();
    }

    internal UiResult<TokenOwner> RetainRuntimeAnchor()
    {
        UiResult<CallLease> leaseResult = _runtimeAnchor.AcquireLease();
        if (leaseResult.IsFailure)
        {
            return UiResult<TokenOwner>.Failure(leaseResult.Error!);
        }

        using CallLease lease = leaseResult.Value;
        return UiCommand.RetainOwner(Api, lease.Token);
    }

    private protected UiResult<UiProperty<T>> GetProperty<T>(
        PropertyGetter getter,
        Func<TokenOwner, UiProperty<T>> wrap)
    {
        UiResult<CallLease> leaseResult = _owner.AcquireLease();
        if (leaseResult.IsFailure)
        {
            return UiResult<UiProperty<T>>.Failure(leaseResult.Error!);
        }

        using CallLease lease = leaseResult.Value;
        var propertyOwner = new TokenOwner(Api);
        ulong propertyToken = 0;
        try
        {
            uint status = getter(Api, lease.Token, out propertyToken);
            if (propertyToken != 0)
            {
                propertyOwner.Adopt(propertyToken);
            }

            if (status != UiStatus.Ok || propertyToken == 0)
            {
                propertyOwner.Dispose();
                return UiResult<UiProperty<T>>.Failure(
                    new UiErrorSnapshot(status == UiStatus.Ok ? UiStatus.InternalError : status));
            }

            return UiResult<UiProperty<T>>.Success(wrap(propertyOwner));
        }
        catch
        {
            propertyOwner.Dispose();
            return UiResult<UiProperty<T>>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    private protected UiWidget WrapOwnedWidget(TokenOwner childOwner, TokenOwner runtimeAnchor) =>
        new GenericWidget(childOwner, runtimeAnchor, _callbackHandlers);

    private protected UiResult<bool> ValidateChild(UiWidget? child)
    {
        if (child is null)
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        return ReferenceEquals(Api, child.Api)
            ? UiResult<bool>.Success(true)
            : UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.WrongRuntime));
    }

    private protected static byte[] EncodeUtf8(string? value)
    {
        if (value is null)
        {
            throw new ArgumentNullException(nameof(value));
        }

        return StrictUtf8.GetBytes(value);
    }

    private protected delegate uint PropertyGetter(INativeApi api, ulong widget, out ulong property);

    private sealed class GenericWidget : UiWidget
    {
        internal GenericWidget(TokenOwner owner, TokenOwner runtimeAnchor, CallbackHandlerState callbackHandlers)
            : base(owner, runtimeAnchor, callbackHandlers)
        {
        }
    }
}

public abstract class UiContainer : UiWidget
{
    internal UiContainer(TokenOwner owner, TokenOwner runtimeAnchor, CallbackHandlerState callbackHandlers)
        : base(owner, runtimeAnchor, callbackHandlers)
    {
    }

    public UiResult<bool> TryInsert(UiWidget child, ulong index)
    {
        UiResult<bool> valid = ValidateChild(child);
        if (valid.IsFailure)
        {
            return valid;
        }

        UiResult<CallLease> parentLeaseResult = Owner.AcquireLease();
        if (parentLeaseResult.IsFailure)
        {
            return UiResult<bool>.Failure(parentLeaseResult.Error!);
        }

        using CallLease parentLease = parentLeaseResult.Value;
        UiResult<CallLease> childLeaseResult = child.Owner.AcquireLease();
        if (childLeaseResult.IsFailure)
        {
            return UiResult<bool>.Failure(childLeaseResult.Error!);
        }

        using CallLease childLease = childLeaseResult.Value;
        return CallError(static (INativeApi api, ulong parent, ulong childToken, ulong indexValue, out ulong error) =>
            api.ContainerInsert(parent, childToken, indexValue, out error),
            parentLease.Token, childLease.Token, index);
    }

    public UiResult<bool> TryRemove(UiWidget child)
    {
        UiResult<bool> valid = ValidateChild(child);
        if (valid.IsFailure)
        {
            return valid;
        }

        UiResult<CallLease> parentLeaseResult = Owner.AcquireLease();
        if (parentLeaseResult.IsFailure)
        {
            return UiResult<bool>.Failure(parentLeaseResult.Error!);
        }

        using CallLease parentLease = parentLeaseResult.Value;
        UiResult<CallLease> childLeaseResult = child.Owner.AcquireLease();
        if (childLeaseResult.IsFailure)
        {
            return UiResult<bool>.Failure(childLeaseResult.Error!);
        }

        using CallLease childLease = childLeaseResult.Value;
        var preparedError = new PreparedError(Api);
        try
        {
            uint status = Api.ContainerRemove(parentLease.Token, childLease.Token, out ulong error);
            return status == UiStatus.Ok
                ? UiResult<bool>.Success(true)
                : UiResult<bool>.Failure(preparedError.Capture(status, error));
        }
        catch
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    public UiResult<bool> TryMove(ulong fromIndex, ulong toIndex)
    {
        UiResult<CallLease> leaseResult = Owner.AcquireLease();
        if (leaseResult.IsFailure)
        {
            return UiResult<bool>.Failure(leaseResult.Error!);
        }

        using CallLease lease = leaseResult.Value;
        var preparedError = new PreparedError(Api);
        try
        {
            uint status = Api.ContainerMove(lease.Token, fromIndex, toIndex, out ulong error);
            return status == UiStatus.Ok
                ? UiResult<bool>.Success(true)
                : UiResult<bool>.Failure(preparedError.Capture(status, error));
        }
        catch
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    public UiResult<ulong> TryGetChildCount()
    {
        UiResult<CallLease> leaseResult = Owner.AcquireLease();
        if (leaseResult.IsFailure)
        {
            return UiResult<ulong>.Failure(leaseResult.Error!);
        }

        using CallLease lease = leaseResult.Value;
        try
        {
            uint status = Api.ContainerChildCount(lease.Token, out ulong count);
            return status == UiStatus.Ok
                ? UiResult<ulong>.Success(count)
                : UiResult<ulong>.Failure(new UiErrorSnapshot(status));
        }
        catch
        {
            return UiResult<ulong>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    public UiResult<UiWidget> TryGetChildAt(ulong index)
    {
        UiResult<CallLease> leaseResult = Owner.AcquireLease();
        if (leaseResult.IsFailure)
        {
            return UiResult<UiWidget>.Failure(leaseResult.Error!);
        }

        using CallLease lease = leaseResult.Value;
        var childOwner = new TokenOwner(Api);
        UiResult<TokenOwner> anchorResult = RetainRuntimeAnchor();
        if (anchorResult.IsFailure)
        {
            childOwner.Dispose();
            return UiResult<UiWidget>.Failure(anchorResult.Error!);
        }

        TokenOwner runtimeAnchor = anchorResult.Value;
        ulong child = 0;
        var preparedError = new PreparedError(Api);
        try
        {
            uint status = Api.ContainerChildAt(lease.Token, index, out child, out ulong error);
            if (child != 0)
            {
                childOwner.Adopt(child);
            }

            if (status != UiStatus.Ok || child == 0)
            {
                childOwner.Dispose();
                runtimeAnchor.Dispose();
                return UiResult<UiWidget>.Failure(preparedError.Capture(status, error));
            }

            return UiResult<UiWidget>.Success(WrapOwnedWidget(childOwner, runtimeAnchor));
        }
        catch
        {
            childOwner.Dispose();
            runtimeAnchor.Dispose();
            return UiResult<UiWidget>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    private protected UiResult<bool> CallError(
        ContainerErrorCall call,
        ulong parent,
        ulong child,
        ulong index)
    {
        var preparedError = new PreparedError(Api);
        try
        {
            uint status = call(Api, parent, child, index, out ulong error);
            return status == UiStatus.Ok
                ? UiResult<bool>.Success(true)
                : UiResult<bool>.Failure(preparedError.Capture(status, error));
        }
        catch
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    private protected delegate uint ContainerErrorCall(
        INativeApi api, ulong parent, ulong child, ulong index, out ulong error);
}

public sealed class UiWindow : UiContainer
{
    private UiWindow(TokenOwner owner, TokenOwner runtimeAnchor, CallbackHandlerState callbackHandlers)
        : base(owner, runtimeAnchor, callbackHandlers)
    {
    }

    public static UiResult<UiWindow> TryCreate(UiRuntime runtime) =>
        Create(runtime, static (INativeApi api, ulong handle, byte[]? _, out ulong control, out ulong error) =>
            api.WindowCreate(handle, out control, out error),
            static (owner, anchor, handlers) => new UiWindow(owner, anchor, handlers));

    public UiResult<UiProperty<string>> TryGetTitleProperty() => GetProperty<string>(
        static (INativeApi api, ulong window, out ulong property) => api.WindowTitleProperty(window, out property),
        owner => UiProperty<string>.FromOwned(owner, CallbackHandlers));

    public UiResult<bool> TryOpen() => CallWindowPeer(Api.WindowOpen);

    public UiResult<bool> TryCloseWindow() => CallWindowPeer(Api.WindowClose);

    public UiResult<UiSubscription> TrySubscribeCloseRequest(Action<UiWindowCloseRequest> callback)
    {
        if (callback is null)
        {
            return UiResult<UiSubscription>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        UiResult<CallLease> windowLeaseResult = Owner.AcquireLease();
        if (windowLeaseResult.IsFailure)
        {
            return UiResult<UiSubscription>.Failure(windowLeaseResult.Error!);
        }

        UiResult<TokenOwner> callbackAnchorResult = RetainRuntimeAnchor();
        if (callbackAnchorResult.IsFailure)
        {
            windowLeaseResult.Value.Dispose();
            return UiResult<UiSubscription>.Failure(callbackAnchorResult.Error!);
        }

        TokenOwner callbackAnchor = callbackAnchorResult.Value;
        TokenOwner subscriptionOwner;
        UiWindowCloseCallbackRoot callbackRoot;
        GCHandle callbackRootHandle = default;
        PreparedError preparedError;
        try
        {
            subscriptionOwner = new TokenOwner(Api);
            callbackRoot = new UiWindowCloseCallbackRoot(Api, callbackAnchor, CallbackHandlers, callback);
            callbackRootHandle = GCHandle.Alloc(callbackRoot);
            preparedError = new PreparedError(Api);
        }
        catch (OutOfMemoryException)
        {
            if (callbackRootHandle.IsAllocated) callbackRootHandle.Free();
            callbackAnchor.Dispose();
            windowLeaseResult.Value.Dispose();
            return UiResult<UiSubscription>.Failure(new UiErrorSnapshot(UiStatus.OutOfMemory));
        }

        using CallLease windowLease = windowLeaseResult.Value;
        uint status;
        ulong subscriptionToken;
        ulong errorToken;
        try
        {
            status = Api.WindowCloseRequestSubscribe(
                windowLease.Token,
                WindowCloseCallbackEntryPoints.CallbackPointer,
                GCHandle.ToIntPtr(callbackRootHandle),
                WindowCloseCallbackEntryPoints.DestroyPointer,
                out subscriptionToken,
                out errorToken);
        }
        catch
        {
            callbackRootHandle.Free();
            subscriptionOwner.Dispose();
            callbackAnchor.Dispose();
            return UiResult<UiSubscription>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }

        if (status != UiStatus.Ok)
        {
            callbackRootHandle.Free();
            if (subscriptionToken != 0) subscriptionOwner.Adopt(subscriptionToken);
            subscriptionOwner.Dispose();
            callbackAnchor.Dispose();
            return UiResult<UiSubscription>.Failure(preparedError.Capture(status, errorToken));
        }

        if (subscriptionToken == 0)
        {
            callbackRootHandle.Free();
            subscriptionOwner.Dispose();
            callbackAnchor.Dispose();
            return UiResult<UiSubscription>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }

        subscriptionOwner.Adopt(subscriptionToken);
        return UiResult<UiSubscription>.Success(new UiSubscription(subscriptionOwner));
    }

    private UiResult<bool> CallWindowPeer(WindowPeerCall call)
    {
        UiResult<CallLease> leaseResult = Owner.AcquireLease();
        if (leaseResult.IsFailure)
        {
            return UiResult<bool>.Failure(leaseResult.Error!);
        }

        using CallLease lease = leaseResult.Value;
        var preparedError = new PreparedError(Api);
        try
        {
            uint status = call(lease.Token, out ulong error);
            return status == UiStatus.Ok
                ? UiResult<bool>.Success(true)
                : UiResult<bool>.Failure(preparedError.Capture(status, error));
        }
        catch
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    private delegate uint WindowPeerCall(ulong window, out ulong error);

    public UiResult<UiWidget?> TryGetContent()
    {
        UiResult<CallLease> leaseResult = Owner.AcquireLease();
        if (leaseResult.IsFailure)
        {
            return UiResult<UiWidget?>.Failure(leaseResult.Error!);
        }

        using CallLease lease = leaseResult.Value;
        var childOwner = new TokenOwner(Api);
        UiResult<TokenOwner> anchorResult = RetainRuntimeAnchor();
        if (anchorResult.IsFailure)
        {
            childOwner.Dispose();
            return UiResult<UiWidget?>.Failure(anchorResult.Error!);
        }

        TokenOwner runtimeAnchor = anchorResult.Value;
        ulong child = 0;
        var preparedError = new PreparedError(Api);
        try
        {
            uint status = Api.WindowContent(lease.Token, out child, out ulong error);
            if (child != 0)
            {
                childOwner.Adopt(child);
            }

            if (status != UiStatus.Ok)
            {
                childOwner.Dispose();
                runtimeAnchor.Dispose();
                return UiResult<UiWidget?>.Failure(preparedError.Capture(status, error));
            }

            if (child == 0)
            {
                childOwner.Dispose();
                runtimeAnchor.Dispose();
                return UiResult<UiWidget?>.Success(null);
            }

            return UiResult<UiWidget?>.Success(WrapOwnedWidget(childOwner, runtimeAnchor));
        }
        catch
        {
            childOwner.Dispose();
            runtimeAnchor.Dispose();
            return UiResult<UiWidget?>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    public UiResult<bool> TrySetContent(UiWidget child)
    {
        UiResult<bool> valid = ValidateChild(child);
        if (valid.IsFailure)
        {
            return valid;
        }

        UiResult<CallLease> windowLeaseResult = Owner.AcquireLease();
        if (windowLeaseResult.IsFailure)
        {
            return UiResult<bool>.Failure(windowLeaseResult.Error!);
        }

        using CallLease windowLease = windowLeaseResult.Value;
        UiResult<CallLease> childLeaseResult = child.Owner.AcquireLease();
        if (childLeaseResult.IsFailure)
        {
            return UiResult<bool>.Failure(childLeaseResult.Error!);
        }

        using CallLease childLease = childLeaseResult.Value;
        var preparedError = new PreparedError(Api);
        try
        {
            uint status = Api.WindowSetContent(windowLease.Token, childLease.Token, out ulong error);
            return status == UiStatus.Ok
                ? UiResult<bool>.Success(true)
                : UiResult<bool>.Failure(preparedError.Capture(status, error));
        }
        catch
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    public UiResult<bool> TryClearContent()
    {
        UiResult<CallLease> leaseResult = Owner.AcquireLease();
        if (leaseResult.IsFailure)
        {
            return UiResult<bool>.Failure(leaseResult.Error!);
        }

        using CallLease lease = leaseResult.Value;
        var preparedError = new PreparedError(Api);
        try
        {
            uint status = Api.WindowClearContent(lease.Token, out ulong error);
            return status == UiStatus.Ok
                ? UiResult<bool>.Success(true)
                : UiResult<bool>.Failure(preparedError.Capture(status, error));
        }
        catch
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    internal static UiResult<TControl> Create<TControl>(
        UiRuntime runtime,
        CreateNativeControl create,
        Func<TokenOwner, TokenOwner, CallbackHandlerState, TControl> wrap,
        byte[]? initial = null)
    {
        if (runtime is null)
        {
            return UiResult<TControl>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        INativeApi api = runtime.Owner.Api;
        PreparedError preparedError;
        try
        {
            preparedError = new PreparedError(api);
        }
        catch (OutOfMemoryException)
        {
            return UiResult<TControl>.Failure(new UiErrorSnapshot(UiStatus.OutOfMemory));
        }

        UiResult<CallLease> runtimeLeaseResult = runtime.Owner.AcquireLease();
        if (runtimeLeaseResult.IsFailure)
        {
            return UiResult<TControl>.Failure(runtimeLeaseResult.Error!);
        }

        using CallLease runtimeLease = runtimeLeaseResult.Value;
        UiResult<TokenOwner> anchorResult = UiCommand.RetainOwner(api, runtimeLease.Token);
        if (anchorResult.IsFailure)
        {
            return UiResult<TControl>.Failure(anchorResult.Error!);
        }

        TokenOwner anchor = anchorResult.Value;
        var owner = new TokenOwner(api);
        ulong control = 0;
        try
        {
            uint status = create(api, runtimeLease.Token, initial, out control, out ulong error);
            if (control != 0)
            {
                owner.Adopt(control);
            }

            if (status != UiStatus.Ok || control == 0)
            {
                owner.Dispose();
                anchor.Dispose();
                return UiResult<TControl>.Failure(status == UiStatus.Ok
                    ? new UiErrorSnapshot(UiStatus.InternalError)
                    : preparedError.Capture(status, error));
            }

            return UiResult<TControl>.Success(wrap(owner, anchor, runtime.CallbackHandlers));
        }
        catch
        {
            owner.Dispose();
            anchor.Dispose();
            return UiResult<TControl>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    internal delegate uint CreateNativeControl(
        INativeApi api, ulong runtime, byte[]? initial, out ulong control, out ulong error);

}

public sealed class UiStackPanel : UiContainer
{
    private UiStackPanel(TokenOwner owner, TokenOwner runtimeAnchor, CallbackHandlerState callbackHandlers)
        : base(owner, runtimeAnchor, callbackHandlers)
    {
    }

    public static UiResult<UiStackPanel> TryCreate(UiRuntime runtime) =>
        UiWindow.Create(runtime, static (INativeApi api, ulong handle, byte[]? _, out ulong control, out ulong error) =>
            api.StackPanelCreate(handle, out control, out error),
            static (owner, anchor, handlers) => new UiStackPanel(owner, anchor, handlers));

    public UiResult<UiControlOrientation> TryGetOrientation()
    {
        UiResult<CallLease> leaseResult = Owner.AcquireLease();
        if (leaseResult.IsFailure)
        {
            return UiResult<UiControlOrientation>.Failure(leaseResult.Error!);
        }

        using CallLease lease = leaseResult.Value;
        try
        {
            uint status = Api.StackPanelOrientation(lease.Token, out uint orientation);
            if (status != UiStatus.Ok)
            {
                return UiResult<UiControlOrientation>.Failure(new UiErrorSnapshot(status));
            }

            return Enum.IsDefined(typeof(UiControlOrientation), orientation)
                ? UiResult<UiControlOrientation>.Success((UiControlOrientation)orientation)
                : UiResult<UiControlOrientation>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
        catch
        {
            return UiResult<UiControlOrientation>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    public UiResult<bool> TrySetOrientation(UiControlOrientation orientation)
    {
        if (!Enum.IsDefined(orientation))
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        UiResult<CallLease> leaseResult = Owner.AcquireLease();
        if (leaseResult.IsFailure)
        {
            return UiResult<bool>.Failure(leaseResult.Error!);
        }

        using CallLease lease = leaseResult.Value;
        var preparedError = new PreparedError(Api);
        try
        {
            uint status = Api.StackPanelSetOrientation(lease.Token, (uint)orientation, out ulong error);
            return status == UiStatus.Ok
                ? UiResult<bool>.Success(true)
                : UiResult<bool>.Failure(preparedError.Capture(status, error));
        }
        catch
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    public UiResult<bool> TrySetChildAlignment(UiWidget child, UiChildAlignment alignment)
    {
        if (!Enum.IsDefined(alignment))
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        UiResult<bool> valid = ValidateChild(child);
        if (valid.IsFailure)
        {
            return valid;
        }

        UiResult<CallLease> panelLeaseResult = Owner.AcquireLease();
        if (panelLeaseResult.IsFailure)
        {
            return UiResult<bool>.Failure(panelLeaseResult.Error!);
        }

        using CallLease panelLease = panelLeaseResult.Value;
        UiResult<CallLease> childLeaseResult = child.Owner.AcquireLease();
        if (childLeaseResult.IsFailure)
        {
            return UiResult<bool>.Failure(childLeaseResult.Error!);
        }

        using CallLease childLease = childLeaseResult.Value;
        var preparedError = new PreparedError(Api);
        try
        {
            uint status = Api.StackPanelSetChildAlign(
                panelLease.Token, childLease.Token, (uint)alignment, out ulong error);
            return status == UiStatus.Ok
                ? UiResult<bool>.Success(true)
                : UiResult<bool>.Failure(preparedError.Capture(status, error));
        }
        catch
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }
}

public sealed class UiText : UiWidget
{
    private UiText(TokenOwner owner, TokenOwner runtimeAnchor, CallbackHandlerState callbackHandlers)
        : base(owner, runtimeAnchor, callbackHandlers)
    {
    }

    public static UiResult<UiText> TryCreate(UiRuntime runtime, string initialText)
    {
        byte[] bytes;
        try
        {
            bytes = EncodeUtf8(initialText);
        }
        catch (ArgumentNullException)
        {
            return UiResult<UiText>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }
        catch (EncoderFallbackException)
        {
            return UiResult<UiText>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        return UiWindow.Create(runtime, (INativeApi api, ulong handle, byte[]? _, out ulong control, out ulong error) =>
            api.TextCreate(handle, bytes, out control, out error),
            static (owner, anchor, handlers) => new UiText(owner, anchor, handlers));
    }

    public UiResult<UiProperty<string>> TryGetContentProperty() => GetProperty<string>(
        static (INativeApi api, ulong text, out ulong property) => api.TextContentProperty(text, out property),
        owner => UiProperty<string>.FromOwned(owner, CallbackHandlers));
}

public sealed class UiTextBox : UiWidget
{
    private UiTextBox(TokenOwner owner, TokenOwner runtimeAnchor, CallbackHandlerState callbackHandlers)
        : base(owner, runtimeAnchor, callbackHandlers)
    {
    }

    public static UiResult<UiTextBox> TryCreate(UiRuntime runtime, string initialText)
    {
        byte[] bytes;
        try
        {
            bytes = EncodeUtf8(initialText);
        }
        catch (ArgumentNullException)
        {
            return UiResult<UiTextBox>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }
        catch (EncoderFallbackException)
        {
            return UiResult<UiTextBox>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        return UiWindow.Create(runtime, (INativeApi api, ulong handle, byte[]? _, out ulong control, out ulong error) =>
            api.TextBoxCreate(handle, bytes, out control, out error),
            static (owner, anchor, handlers) => new UiTextBox(owner, anchor, handlers));
    }

    public UiResult<UiProperty<string>> TryGetValueProperty() => GetProperty<string>(
        static (INativeApi api, ulong textBox, out ulong property) => api.TextBoxValueProperty(textBox, out property),
        owner => UiProperty<string>.FromOwned(owner, CallbackHandlers));
}

public sealed class UiButton : UiWidget
{
    private UiButton(TokenOwner owner, TokenOwner runtimeAnchor, CallbackHandlerState callbackHandlers)
        : base(owner, runtimeAnchor, callbackHandlers)
    {
    }

    public static UiResult<UiButton> TryCreate(UiRuntime runtime, string initialLabel)
    {
        byte[] bytes;
        try
        {
            bytes = EncodeUtf8(initialLabel);
        }
        catch (ArgumentNullException)
        {
            return UiResult<UiButton>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }
        catch (EncoderFallbackException)
        {
            return UiResult<UiButton>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        return UiWindow.Create(runtime, (INativeApi api, ulong handle, byte[]? _, out ulong control, out ulong error) =>
            api.ButtonCreate(handle, bytes, out control, out error),
            static (owner, anchor, handlers) => new UiButton(owner, anchor, handlers));
    }

    public UiResult<UiProperty<string>> TryGetLabelProperty() => GetProperty<string>(
        static (INativeApi api, ulong button, out ulong property) => api.ButtonLabelProperty(button, out property),
        owner => UiProperty<string>.FromOwned(owner, CallbackHandlers));

    public UiResult<bool> TrySetCommand(UiCommand? command)
    {
        if (command is not null && !ReferenceEquals(Api, command.Owner.Api))
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.WrongRuntime));
        }

        UiResult<CallLease> buttonLeaseResult = Owner.AcquireLease();
        if (buttonLeaseResult.IsFailure)
        {
            return UiResult<bool>.Failure(buttonLeaseResult.Error!);
        }

        using CallLease buttonLease = buttonLeaseResult.Value;
        if (command is null)
        {
            return SetCommand(buttonLease.Token, 0);
        }

        UiResult<CallLease> commandLeaseResult = command.Owner.AcquireLease();
        if (commandLeaseResult.IsFailure)
        {
            return UiResult<bool>.Failure(commandLeaseResult.Error!);
        }

        using CallLease commandLease = commandLeaseResult.Value;
        return SetCommand(buttonLease.Token, commandLease.Token);
    }

    public UiResult<UiCommand?> TryGetCommand()
    {
        UiResult<CallLease> leaseResult = Owner.AcquireLease();
        if (leaseResult.IsFailure)
        {
            return UiResult<UiCommand?>.Failure(leaseResult.Error!);
        }

        using CallLease lease = leaseResult.Value;
        var owner = new TokenOwner(Api);
        UiResult<TokenOwner> anchorResult = RetainRuntimeAnchor();
        if (anchorResult.IsFailure)
        {
            owner.Dispose();
            return UiResult<UiCommand?>.Failure(anchorResult.Error!);
        }

        TokenOwner anchor = anchorResult.Value;
        ulong commandToken = 0;
        var preparedError = new PreparedError(Api);
        try
        {
            uint status = Api.ButtonCommand(lease.Token, out commandToken, out ulong error);
            if (commandToken != 0)
            {
                owner.Adopt(commandToken);
            }

            if (status != UiStatus.Ok)
            {
                owner.Dispose();
                anchor.Dispose();
                return UiResult<UiCommand?>.Failure(preparedError.Capture(status, error));
            }

            if (commandToken == 0)
            {
                owner.Dispose();
                anchor.Dispose();
                return UiResult<UiCommand?>.Success(null);
            }

            return UiResult<UiCommand?>.Success(
                UiCommand.FromOwned(owner, anchor, CallbackHandlers));
        }
        catch
        {
            owner.Dispose();
            anchor.Dispose();
            return UiResult<UiCommand?>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    private UiResult<bool> SetCommand(ulong button, ulong command)
    {
        var preparedError = new PreparedError(Api);
        try
        {
            uint status = Api.ButtonSetCommand(button, command, out ulong error);
            return status == UiStatus.Ok
                ? UiResult<bool>.Success(true)
                : UiResult<bool>.Failure(preparedError.Capture(status, error));
        }
        catch
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }
}
