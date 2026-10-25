using System;
using System.IO;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace FreeMvvm;

internal interface INativeApi
{
    uint RuntimeCreate(out ulong runtime, out ulong error);

    uint RuntimeIsUiThread(ulong runtime, out byte isUiThread);

    uint RuntimeSetUiLoopBridge(ulong runtime, IntPtr wake, IntPtr userdata, IntPtr destroy, out ulong error);

    uint RuntimeDispatchTurn(ulong runtime);

    uint RuntimePost(ulong runtime, IntPtr callback, IntPtr userdata, IntPtr destroy, out ulong error);

    uint PropertyCreateBool(ulong runtime, byte initial, out ulong property, out ulong error);

    uint PropertyCreateI64(ulong runtime, long initial, out ulong property, out ulong error);

    uint PropertyCreateF64(ulong runtime, double initial, out ulong property, out ulong error);

    uint PropertyCreateDecimal(ulong runtime, ulong initial, out ulong property, out ulong error);

    uint PropertyCreateUtf8(ulong runtime, byte[] initial, out ulong property, out ulong error);

    uint PropertyGetBool(ulong property, out byte value);

    uint PropertyGetI64(ulong property, out long value);

    uint PropertyGetF64(ulong property, out double value);

    uint PropertyGetDecimal(ulong property, out ulong value);

    uint PropertyGetUtf8(ulong property, byte[] buffer, ulong capacity, out ulong length);

    uint PropertySetBool(ulong property, byte value, out ulong error);

    uint PropertySetI64(ulong property, long value, out ulong error);

    uint PropertySetF64(ulong property, double value, out ulong error);

    uint PropertySetDecimal(ulong property, ulong value, out ulong error);

    uint PropertySetUtf8(ulong property, byte[] value, out ulong error);

    uint PropertySubscribe(ulong property, IntPtr callback, IntPtr userdata, IntPtr destroy, out ulong subscription, out ulong error);

    uint SubscriptionCancel(ulong subscription, out ulong error);

    uint BindingCreate(ulong source, ulong target, uint mode, out ulong binding, out ulong error);

    uint BindingCancel(ulong binding, out ulong error);

    uint BindingErrorSnapshot(ulong binding, out ulong errorSnapshot, out ulong rejectedProperty, out uint direction);

    uint BindingSubscribeError(ulong binding, IntPtr callback, IntPtr userdata, IntPtr destroy, out ulong subscription, out ulong error);

    uint CommandCreateAsync(ulong runtime, ulong eligibility, IntPtr start, IntPtr userdata, IntPtr destroy, out ulong command, out ulong error);

    uint CommandExecute(ulong command, out ulong operationId, out ulong error);

    uint CommandCancel(ulong command, ulong operationId, out ulong error);

    uint CommandClose(ulong command, out ulong error);

    uint CommandStateProperty(ulong command, out ulong property);

    uint CommandCanExecuteProperty(ulong command, out ulong property);

    uint CommandFailureSnapshot(ulong command, out ulong errorSnapshot);

    uint CommandCancelTokenRequested(ulong token, out byte requested);

    uint CommandCompleteSuccess(ulong command, ulong operationId);

    unsafe uint CommandCompleteFailureUtf8(ulong command, ulong operationId, uint domain, uint code, byte* message, ulong length);

    uint CommandCompleteCancelled(ulong command, ulong operationId);

    uint WindowCreate(ulong runtime, out ulong window, out ulong error);

    uint StackPanelCreate(ulong runtime, out ulong panel, out ulong error);

    uint TextCreate(ulong runtime, byte[] initial, out ulong text, out ulong error);

    uint TextBoxCreate(ulong runtime, byte[] initial, out ulong textBox, out ulong error);

    uint ButtonCreate(ulong runtime, byte[] initial, out ulong button, out ulong error);

    uint WidgetIsEnabledProperty(ulong widget, out ulong property);

    uint WidgetAccessibleNameProperty(ulong widget, out ulong property);

    uint WidgetRequestFocus(ulong widget, out ulong error);

    uint WindowContent(ulong window, out ulong child, out ulong error);

    uint WindowSetContent(ulong window, ulong child, out ulong error);

    uint WindowClearContent(ulong window, out ulong error);

    uint WindowTitleProperty(ulong window, out ulong property);

    uint WindowOpen(ulong window, out ulong error);

    uint WindowClose(ulong window, out ulong error);

    uint WindowCloseRequestSubscribe(ulong window, IntPtr callback, IntPtr userdata, IntPtr destroy, out ulong subscription, out ulong error);

    uint WindowCloseRequestDefer(ulong borrowedRequest, out ulong ownedRequest, out ulong error);

    uint WindowCloseRequestResolve(ulong request, uint decision, out ulong error);

    uint ContainerInsert(ulong container, ulong child, ulong index, out ulong error);

    uint ContainerRemove(ulong container, ulong child, out ulong error);

    uint ContainerMove(ulong container, ulong fromIndex, ulong toIndex, out ulong error);

    uint ContainerChildCount(ulong container, out ulong count);

    uint ContainerChildAt(ulong container, ulong index, out ulong child, out ulong error);

    uint StackPanelOrientation(ulong panel, out uint orientation);

    uint StackPanelSetOrientation(ulong panel, uint orientation, out ulong error);

    uint StackPanelSetChildAlign(ulong panel, ulong child, uint alignment, out ulong error);

    uint TextContentProperty(ulong text, out ulong property);

    uint TextBoxValueProperty(ulong textBox, out ulong property);

    uint ButtonLabelProperty(ulong button, out ulong property);

    uint ButtonSetCommand(ulong button, ulong command, out ulong error);

    uint ButtonCommand(ulong button, out ulong command, out ulong error);

    uint HandleRetain(ulong borrowed, out ulong owned);

    uint RuntimeRequestShutdown(ulong runtime, uint mode, out ulong request, out ulong error);

    uint ShutdownRequestQuery(ulong request, out uint state, out uint reason, out ulong error);

    uint RuntimeShutdown(ulong runtime, out ulong error);

    uint HandleRelease(ulong owned);

    uint ErrorStatus(ulong error, out uint status);

    uint ErrorCode(ulong error, out uint domain, out uint code);

    unsafe uint ErrorMessageUtf8(ulong error, byte* buffer, ulong capacity, out ulong length);

    uint GetVersion(out uint major, out uint minor, out uint patch);
}

internal sealed class NativeRuntimeApi : INativeApi
{
    internal static NativeRuntimeApi Open(string? nativeArtifactPath)
    {
        NativeLibraryLifetime.Configure(nativeArtifactPath);
        uint status = NativeMethods.GetVersion(out uint major, out _, out _);
        if (status != 0 || major != 1)
        {
            throw new NativeApiException(status != 0 ? status : UiStatus.Unsupported);
        }

        return new NativeRuntimeApi();
    }

    public uint RuntimeCreate(out ulong runtime, out ulong error) =>
        NativeMethods.RuntimeCreate(out runtime, out error);

    public uint RuntimeIsUiThread(ulong runtime, out byte isUiThread) =>
        NativeMethods.RuntimeIsUiThread(runtime, out isUiThread);

    public uint RuntimeSetUiLoopBridge(ulong runtime, IntPtr wake, IntPtr userdata, IntPtr destroy, out ulong error) =>
        NativeMethods.RuntimeSetUiLoopBridge(runtime, wake, userdata, destroy, out error);

    public uint RuntimeDispatchTurn(ulong runtime) => NativeMethods.RuntimeDispatchTurn(runtime);

    public uint RuntimePost(ulong runtime, IntPtr callback, IntPtr userdata, IntPtr destroy, out ulong error) =>
        NativeMethods.RuntimePost(runtime, callback, userdata, destroy, out error);

    public uint PropertyCreateBool(ulong runtime, byte initial, out ulong property, out ulong error) =>
        NativeMethods.PropertyCreateBool(runtime, initial, out property, out error);

    public uint PropertyCreateI64(ulong runtime, long initial, out ulong property, out ulong error) =>
        NativeMethods.PropertyCreateI64(runtime, initial, out property, out error);

    public uint PropertyCreateF64(ulong runtime, double initial, out ulong property, out ulong error) =>
        NativeMethods.PropertyCreateF64(runtime, initial, out property, out error);

    public uint PropertyCreateDecimal(ulong runtime, ulong initial, out ulong property, out ulong error) =>
        NativeMethods.PropertyCreateDecimal(runtime, initial, out property, out error);

    public unsafe uint PropertyCreateUtf8(ulong runtime, byte[] initial, out ulong property, out ulong error)
    {
        fixed (byte* bytes = initial)
        {
            return NativeMethods.PropertyCreateUtf8(runtime, bytes, (ulong)initial.Length, out property, out error);
        }
    }

    public uint PropertyGetBool(ulong property, out byte value) =>
        NativeMethods.PropertyGetBool(property, out value);

    public uint PropertyGetI64(ulong property, out long value) =>
        NativeMethods.PropertyGetI64(property, out value);

    public uint PropertyGetF64(ulong property, out double value) =>
        NativeMethods.PropertyGetF64(property, out value);

    public uint PropertyGetDecimal(ulong property, out ulong value) =>
        NativeMethods.PropertyGetDecimal(property, out value);

    public unsafe uint PropertyGetUtf8(ulong property, byte[] buffer, ulong capacity, out ulong length)
    {
        fixed (byte* bytes = buffer)
        {
            return NativeMethods.PropertyGetUtf8(property, bytes, capacity, out length);
        }
    }

    public uint PropertySetBool(ulong property, byte value, out ulong error) =>
        NativeMethods.PropertySetBool(property, value, out error);

    public uint PropertySetI64(ulong property, long value, out ulong error) =>
        NativeMethods.PropertySetI64(property, value, out error);

    public uint PropertySetF64(ulong property, double value, out ulong error) =>
        NativeMethods.PropertySetF64(property, value, out error);

    public uint PropertySetDecimal(ulong property, ulong value, out ulong error) =>
        NativeMethods.PropertySetDecimal(property, value, out error);

    public unsafe uint PropertySetUtf8(ulong property, byte[] value, out ulong error)
    {
        fixed (byte* bytes = value)
        {
            return NativeMethods.PropertySetUtf8(property, bytes, (ulong)value.Length, out error);
        }
    }

    public uint PropertySubscribe(ulong property, IntPtr callback, IntPtr userdata, IntPtr destroy, out ulong subscription, out ulong error) =>
        NativeMethods.PropertySubscribe(property, callback, userdata, destroy, out subscription, out error);

    public uint SubscriptionCancel(ulong subscription, out ulong error) =>
        NativeMethods.SubscriptionCancel(subscription, out error);

    public uint BindingCreate(ulong source, ulong target, uint mode, out ulong binding, out ulong error) =>
        NativeMethods.BindingCreate(source, target, mode, out binding, out error);

    public uint BindingCancel(ulong binding, out ulong error) =>
        NativeMethods.BindingCancel(binding, out error);

    public uint BindingErrorSnapshot(ulong binding, out ulong errorSnapshot, out ulong rejectedProperty, out uint direction) =>
        NativeMethods.BindingErrorSnapshot(binding, out errorSnapshot, out rejectedProperty, out direction);

    public uint BindingSubscribeError(ulong binding, IntPtr callback, IntPtr userdata, IntPtr destroy, out ulong subscription, out ulong error) =>
        NativeMethods.BindingSubscribeError(binding, callback, userdata, destroy, out subscription, out error);

    public uint CommandCreateAsync(ulong runtime, ulong eligibility, IntPtr start, IntPtr userdata, IntPtr destroy, out ulong command, out ulong error) =>
        NativeMethods.CommandCreateAsync(runtime, eligibility, start, userdata, destroy, out command, out error);

    public uint CommandExecute(ulong command, out ulong operationId, out ulong error) =>
        NativeMethods.CommandExecute(command, out operationId, out error);

    public uint CommandCancel(ulong command, ulong operationId, out ulong error) =>
        NativeMethods.CommandCancel(command, operationId, out error);

    public uint CommandClose(ulong command, out ulong error) => NativeMethods.CommandClose(command, out error);

    public uint CommandStateProperty(ulong command, out ulong property) => NativeMethods.CommandStateProperty(command, out property);

    public uint CommandCanExecuteProperty(ulong command, out ulong property) => NativeMethods.CommandCanExecuteProperty(command, out property);

    public uint CommandFailureSnapshot(ulong command, out ulong errorSnapshot) => NativeMethods.CommandFailureSnapshot(command, out errorSnapshot);

    public uint CommandCancelTokenRequested(ulong token, out byte requested) => NativeMethods.CommandCancelTokenRequested(token, out requested);

    public uint CommandCompleteSuccess(ulong command, ulong operationId) => NativeMethods.CommandCompleteSuccess(command, operationId);

    public unsafe uint CommandCompleteFailureUtf8(ulong command, ulong operationId, uint domain, uint code, byte* message, ulong length) =>
        NativeMethods.CommandCompleteFailureUtf8(command, operationId, domain, code, message, length);

    public uint CommandCompleteCancelled(ulong command, ulong operationId) => NativeMethods.CommandCompleteCancelled(command, operationId);

    public uint WindowCreate(ulong runtime, out ulong window, out ulong error) => NativeMethods.WindowCreate(runtime, out window, out error);

    public uint StackPanelCreate(ulong runtime, out ulong panel, out ulong error) => NativeMethods.StackPanelCreate(runtime, out panel, out error);

    public unsafe uint TextCreate(ulong runtime, byte[] initial, out ulong text, out ulong error)
    {
        fixed (byte* bytes = initial)
        {
            return NativeMethods.TextCreate(runtime, bytes, (ulong)initial.Length, out text, out error);
        }
    }

    public unsafe uint TextBoxCreate(ulong runtime, byte[] initial, out ulong textBox, out ulong error)
    {
        fixed (byte* bytes = initial)
        {
            return NativeMethods.TextBoxCreate(runtime, bytes, (ulong)initial.Length, out textBox, out error);
        }
    }

    public unsafe uint ButtonCreate(ulong runtime, byte[] initial, out ulong button, out ulong error)
    {
        fixed (byte* bytes = initial)
        {
            return NativeMethods.ButtonCreate(runtime, bytes, (ulong)initial.Length, out button, out error);
        }
    }

    public uint WidgetIsEnabledProperty(ulong widget, out ulong property) => NativeMethods.WidgetIsEnabledProperty(widget, out property);

    public uint WidgetAccessibleNameProperty(ulong widget, out ulong property) => NativeMethods.WidgetAccessibleNameProperty(widget, out property);

    public uint WidgetRequestFocus(ulong widget, out ulong error) => NativeMethods.WidgetRequestFocus(widget, out error);

    public uint WindowContent(ulong window, out ulong child, out ulong error) => NativeMethods.WindowContent(window, out child, out error);

    public uint WindowSetContent(ulong window, ulong child, out ulong error) => NativeMethods.WindowSetContent(window, child, out error);

    public uint WindowClearContent(ulong window, out ulong error) => NativeMethods.WindowClearContent(window, out error);

    public uint WindowTitleProperty(ulong window, out ulong property) => NativeMethods.WindowTitleProperty(window, out property);

    public uint WindowOpen(ulong window, out ulong error) => NativeMethods.WindowOpen(window, out error);

    public uint WindowClose(ulong window, out ulong error) => NativeMethods.WindowClose(window, out error);

    public uint WindowCloseRequestSubscribe(ulong window, IntPtr callback, IntPtr userdata, IntPtr destroy, out ulong subscription, out ulong error) =>
        NativeMethods.WindowCloseRequestSubscribe(window, callback, userdata, destroy, out subscription, out error);

    public uint WindowCloseRequestDefer(ulong borrowedRequest, out ulong ownedRequest, out ulong error) =>
        NativeMethods.WindowCloseRequestDefer(borrowedRequest, out ownedRequest, out error);

    public uint WindowCloseRequestResolve(ulong request, uint decision, out ulong error) =>
        NativeMethods.WindowCloseRequestResolve(request, decision, out error);

    public uint ContainerInsert(ulong container, ulong child, ulong index, out ulong error) => NativeMethods.ContainerInsert(container, child, index, out error);

    public uint ContainerRemove(ulong container, ulong child, out ulong error) => NativeMethods.ContainerRemove(container, child, out error);

    public uint ContainerMove(ulong container, ulong fromIndex, ulong toIndex, out ulong error) => NativeMethods.ContainerMove(container, fromIndex, toIndex, out error);

    public uint ContainerChildCount(ulong container, out ulong count) => NativeMethods.ContainerChildCount(container, out count);

    public uint ContainerChildAt(ulong container, ulong index, out ulong child, out ulong error) => NativeMethods.ContainerChildAt(container, index, out child, out error);

    public uint StackPanelOrientation(ulong panel, out uint orientation) => NativeMethods.StackPanelOrientation(panel, out orientation);

    public uint StackPanelSetOrientation(ulong panel, uint orientation, out ulong error) => NativeMethods.StackPanelSetOrientation(panel, orientation, out error);

    public uint StackPanelSetChildAlign(ulong panel, ulong child, uint alignment, out ulong error) => NativeMethods.StackPanelSetChildAlign(panel, child, alignment, out error);

    public uint TextContentProperty(ulong text, out ulong property) => NativeMethods.TextContentProperty(text, out property);

    public uint TextBoxValueProperty(ulong textBox, out ulong property) => NativeMethods.TextBoxValueProperty(textBox, out property);

    public uint ButtonLabelProperty(ulong button, out ulong property) => NativeMethods.ButtonLabelProperty(button, out property);

    public uint ButtonSetCommand(ulong button, ulong command, out ulong error) => NativeMethods.ButtonSetCommand(button, command, out error);

    public uint ButtonCommand(ulong button, out ulong command, out ulong error) => NativeMethods.ButtonCommand(button, out command, out error);

    public uint HandleRetain(ulong borrowed, out ulong owned) => NativeMethods.HandleRetain(borrowed, out owned);

    public uint RuntimeRequestShutdown(ulong runtime, uint mode, out ulong request, out ulong error) =>
        NativeMethods.RuntimeRequestShutdown(runtime, mode, out request, out error);

    public uint ShutdownRequestQuery(ulong request, out uint state, out uint reason, out ulong error) =>
        NativeMethods.ShutdownRequestQuery(request, out state, out reason, out error);

    public uint RuntimeShutdown(ulong runtime, out ulong error) =>
        NativeMethods.RuntimeShutdown(runtime, out error);

    public uint HandleRelease(ulong owned) => NativeMethods.HandleRelease(owned);

    public uint ErrorStatus(ulong error, out uint status) => NativeMethods.ErrorStatus(error, out status);

    public uint ErrorCode(ulong error, out uint domain, out uint code) =>
        NativeMethods.ErrorCode(error, out domain, out code);

    public unsafe uint ErrorMessageUtf8(ulong error, byte* buffer, ulong capacity, out ulong length) =>
        NativeMethods.ErrorMessageUtf8(error, buffer, capacity, out length);

    public uint GetVersion(out uint major, out uint minor, out uint patch) =>
        NativeMethods.GetVersion(out major, out minor, out patch);
}

internal sealed class NativeApiException : Exception
{
    internal NativeApiException(uint rawStatus)
        : base($"The native FreeMvvm ABI could not be initialized (status {rawStatus}).")
    {
        RawStatus = rawStatus;
    }

    internal uint RawStatus { get; }
}

internal static class UiStatus
{
    internal const uint Ok = 0;
    internal const uint InvalidArgument = 1;
    internal const uint InvalidHandle = 2;
    internal const uint WrongType = 3;
    internal const uint WrongRuntime = 4;
    internal const uint WrongThread = 5;
    internal const uint Closed = 6;
    internal const uint BufferTooSmall = 7;
    internal const uint OutOfMemory = 8;
    internal const uint InternalError = 9;
    internal const uint Unsupported = 10;
    internal const uint Pending = 14;
}

internal static partial class NativeMethods
{
    private const string LibraryName = "freemvvm_abi_1";

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_runtime_create")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint RuntimeCreate(out ulong runtime, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_runtime_is_ui_thread")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint RuntimeIsUiThread(ulong runtime, out byte isUiThread);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_runtime_set_ui_loop_bridge")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint RuntimeSetUiLoopBridge(
        ulong runtime, IntPtr wake, IntPtr userdata, IntPtr destroy, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_runtime_dispatch_turn")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint RuntimeDispatchTurn(ulong runtime);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_runtime_post")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint RuntimePost(ulong runtime, IntPtr callback, IntPtr userdata, IntPtr destroy, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_property_create_bool")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint PropertyCreateBool(ulong runtime, byte initial, out ulong property, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_property_create_i64")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint PropertyCreateI64(ulong runtime, long initial, out ulong property, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_property_create_f64")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint PropertyCreateF64(ulong runtime, double initial, out ulong property, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_property_create_decimal")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint PropertyCreateDecimal(ulong runtime, ulong initial, out ulong property, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_property_create_utf8")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial uint PropertyCreateUtf8(ulong runtime, byte* initial, ulong byteLength, out ulong property, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_property_get_bool")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint PropertyGetBool(ulong property, out byte value);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_property_get_i64")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint PropertyGetI64(ulong property, out long value);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_property_get_f64")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint PropertyGetF64(ulong property, out double value);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_property_get_decimal")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint PropertyGetDecimal(ulong property, out ulong value);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_property_get_utf8")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial uint PropertyGetUtf8(ulong property, byte* buffer, ulong capacity, out ulong length);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_property_set_bool")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint PropertySetBool(ulong property, byte value, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_property_set_i64")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint PropertySetI64(ulong property, long value, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_property_set_f64")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint PropertySetF64(ulong property, double value, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_property_set_decimal")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint PropertySetDecimal(ulong property, ulong value, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_property_set_utf8")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial uint PropertySetUtf8(ulong property, byte* value, ulong byteLength, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_property_subscribe")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint PropertySubscribe(
        ulong property, IntPtr callback, IntPtr userdata, IntPtr destroy, out ulong subscription, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_subscription_cancel")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint SubscriptionCancel(ulong subscription, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_binding_create")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint BindingCreate(ulong source, ulong target, uint mode, out ulong binding, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_binding_cancel")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint BindingCancel(ulong binding, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_binding_error_snapshot")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint BindingErrorSnapshot(ulong binding, out ulong errorSnapshot, out ulong rejectedProperty, out uint direction);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_binding_subscribe_error")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint BindingSubscribeError(
        ulong binding, IntPtr callback, IntPtr userdata, IntPtr destroy, out ulong subscription, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_command_create_async")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint CommandCreateAsync(ulong runtime, ulong eligibility, IntPtr start, IntPtr userdata, IntPtr destroy, out ulong command, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_command_execute")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint CommandExecute(ulong command, out ulong operationId, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_command_cancel")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint CommandCancel(ulong command, ulong operationId, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_command_close")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint CommandClose(ulong command, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_command_state_property")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint CommandStateProperty(ulong command, out ulong property);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_command_can_execute_property")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint CommandCanExecuteProperty(ulong command, out ulong property);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_command_failure_snapshot")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint CommandFailureSnapshot(ulong command, out ulong errorSnapshot);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_command_cancel_token_requested")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint CommandCancelTokenRequested(ulong token, out byte requested);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_command_complete_success")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint CommandCompleteSuccess(ulong command, ulong operationId);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_command_complete_failure_utf8")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial uint CommandCompleteFailureUtf8(ulong command, ulong operationId, uint domain, uint code, byte* message, ulong length);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_command_complete_cancelled")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint CommandCompleteCancelled(ulong command, ulong operationId);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_window_create")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint WindowCreate(ulong runtime, out ulong window, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_stack_panel_create")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint StackPanelCreate(ulong runtime, out ulong panel, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_text_create")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial uint TextCreate(ulong runtime, byte* initial, ulong length, out ulong text, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_text_box_create")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial uint TextBoxCreate(ulong runtime, byte* initial, ulong length, out ulong textBox, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_button_create")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial uint ButtonCreate(ulong runtime, byte* initial, ulong length, out ulong button, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_widget_is_enabled_property")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint WidgetIsEnabledProperty(ulong widget, out ulong property);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_widget_accessible_name_property")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint WidgetAccessibleNameProperty(ulong widget, out ulong property);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_widget_request_focus")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint WidgetRequestFocus(ulong widget, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_window_content")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint WindowContent(ulong window, out ulong child, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_window_set_content")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint WindowSetContent(ulong window, ulong child, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_window_clear_content")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint WindowClearContent(ulong window, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_window_title_property")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint WindowTitleProperty(ulong window, out ulong property);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_window_open")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint WindowOpen(ulong window, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_window_close")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint WindowClose(ulong window, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_window_close_request_subscribe")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint WindowCloseRequestSubscribe(
        ulong window, IntPtr callback, IntPtr userdata, IntPtr destroy, out ulong subscription, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_window_close_request_defer")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint WindowCloseRequestDefer(ulong borrowedRequest, out ulong ownedRequest, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_window_close_request_resolve")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint WindowCloseRequestResolve(ulong request, uint decision, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_container_insert")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint ContainerInsert(ulong container, ulong child, ulong index, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_container_remove")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint ContainerRemove(ulong container, ulong child, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_container_move")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint ContainerMove(ulong container, ulong fromIndex, ulong toIndex, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_container_child_count")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint ContainerChildCount(ulong container, out ulong count);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_container_child_at")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint ContainerChildAt(ulong container, ulong index, out ulong child, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_stack_panel_orientation")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint StackPanelOrientation(ulong panel, out uint orientation);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_stack_panel_set_orientation")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint StackPanelSetOrientation(ulong panel, uint orientation, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_stack_panel_set_child_align")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint StackPanelSetChildAlign(ulong panel, ulong child, uint alignment, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_text_content_property")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint TextContentProperty(ulong text, out ulong property);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_text_box_value_property")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint TextBoxValueProperty(ulong textBox, out ulong property);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_button_label_property")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint ButtonLabelProperty(ulong button, out ulong property);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_button_set_command")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint ButtonSetCommand(ulong button, ulong command, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_button_command")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint ButtonCommand(ulong button, out ulong command, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_runtime_request_shutdown")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint RuntimeRequestShutdown(ulong runtime, uint mode, out ulong request, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_shutdown_request_query")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint ShutdownRequestQuery(ulong request, out uint state, out uint reason, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_runtime_shutdown")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint RuntimeShutdown(ulong runtime, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_handle_release")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint HandleRelease(ulong owned);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_handle_retain")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint HandleRetain(ulong borrowed, out ulong owned);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_error_status")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint ErrorStatus(ulong error, out uint status);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_error_code")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint ErrorCode(ulong error, out uint domain, out uint code);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_error_message_utf8")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial uint ErrorMessageUtf8(ulong error, byte* buffer, ulong capacity, out ulong length);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_get_version")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint GetVersion(out uint major, out uint minor, out uint patch);
}

internal static class NativeLibraryLifetime
{
    private static readonly object Sync = new();
    private static string? _configuredPath;
    private static nint _libraryHandle;
    private static bool _resolverInstalled;

    internal static void Configure(string? path)
    {
        if (path is not null && !Path.IsPathFullyQualified(path))
        {
            throw new DllNotFoundException("The configured FreeMvvm native artifact path must be absolute.");
        }

        string? fullPath = path is null ? null : Path.GetFullPath(path);
        if (fullPath is not null && !File.Exists(fullPath))
        {
            throw new DllNotFoundException("The configured FreeMvvm native artifact path is not an existing absolute file.");
        }

        lock (Sync)
        {
            if (_resolverInstalled)
            {
                if (!string.Equals(_configuredPath, fullPath, StringComparison.OrdinalIgnoreCase))
                {
                    throw new InvalidOperationException("The process already loaded FreeMvvm from a different native artifact path.");
                }

                return;
            }

            _configuredPath = fullPath;
            NativeLibrary.SetDllImportResolver(typeof(NativeMethods).Assembly, Resolve);
            _resolverInstalled = true;
        }
    }

    private static nint Resolve(string libraryName, System.Reflection.Assembly assembly, DllImportSearchPath? searchPath)
    {
        if (!string.Equals(libraryName, "freemvvm_abi_1", StringComparison.Ordinal))
        {
            return 0;
        }

        lock (Sync)
        {
            if (_libraryHandle != 0)
            {
                return _libraryHandle;
            }

            _libraryHandle = _configuredPath is null
                ? NativeLibrary.Load(libraryName, assembly, searchPath)
                : NativeLibrary.Load(_configuredPath);
            return _libraryHandle;
        }
    }
}
