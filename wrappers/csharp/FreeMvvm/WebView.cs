using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;

namespace FreeMvvm;

/// <summary>
/// Document format accepted by <see cref="WebView"/>.
/// </summary>
public enum WebViewFormat : uint
{
    Html = 1,
    Markdown = 2,
}

/// <summary>
/// Immutable error diagnostic returned across the FreeMVVM C ABI boundary.
/// </summary>
public sealed class UiError
{
    public UiError(uint rawStatus, uint? domain = null, uint? code = null)
    {
        RawStatus = rawStatus;
        Domain = domain;
        Code = code;
    }

    public uint RawStatus { get; }
    public uint? Domain { get; }
    public uint? Code { get; }
}

/// <summary>
/// Exception-free result carrier for void operations.
/// </summary>
public readonly struct Result
{
    private Result(UiError? error)
    {
        Error = error;
    }

    public bool IsSuccess => Error is null;
    public UiError? Error { get; }

    public static Result Success() => new(null);
    public static Result Failure(UiError error) => new(error ?? throw new ArgumentNullException(nameof(error)));
}

/// <summary>
/// Exception-free result carrier for value-returning operations.
/// </summary>
public readonly struct Result<T>
{
    private readonly T? _value;

    private Result(T? value, UiError? error)
    {
        _value = value;
        Error = error;
    }

    public bool IsSuccess => Error is null;
    public T? Value => _value;
    public UiError? Error { get; }

    public static Result<T> Success(T value) => new(value, null);
    public static Result<T> Failure(UiError error) => new(default, error ?? throw new ArgumentNullException(nameof(error)));
}

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
public delegate void UiPostCallback(IntPtr userdata);

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
public delegate void UiDestroyCallback(IntPtr userdata);

/// <summary>
/// Abstraction over the versioned <c>ui_abi_1_*webview*</c> entry points.
/// </summary>
public interface IWebViewNativeApi
{
    uint HandleRetain(ulong handle, out ulong outHandle);
    uint HandleRelease(ulong handle);
    uint ErrorCode(ulong errorHandle, out uint outDomain, out uint outCode);
    uint ControlCreateWebView(ulong runtime, out ulong outWebView, out ulong outError);
    uint WebViewSetSourceUtf8(ulong webview, uint format, byte[] utf8Bytes, ulong byteLen, out ulong outError);
    uint WebViewSetStylesheetUtf8(ulong webview, byte[] utf8Bytes, ulong byteLen, out ulong outError);
    uint WebViewSetViewport(ulong webview, double widthDip, double heightDip, out ulong outError);
    uint WebViewBindSourceProperty(ulong webview, uint format, ulong utf8Property, out ulong outBinding, out ulong outError);
    uint WebViewExposePropertyUtf8(ulong webview, byte[] aliasUtf8, ulong aliasLen, ulong property, byte allowWrite, out ulong outError);
    uint WebViewExposeCommandUtf8(
        ulong webview,
        byte[] aliasUtf8,
        ulong aliasLen,
        UiPostCallback callback,
        IntPtr userdata,
        UiDestroyCallback destroy,
        out ulong outError);
    uint WebViewEvalScriptUtf8(ulong webview, byte[] scriptUtf8, ulong scriptLen, out ulong outError);
    uint WebViewClickAt(ulong webview, double xDip, double yDip, out ulong outError);
    uint WebViewTextContentUtf8(ulong webview, byte[]? buffer, ulong capacity, out ulong outByteLen, out ulong outError);
    uint WebViewRenderRgba(ulong webview, uint widthPx, uint heightPx, byte[]? rgbaBuffer, ulong capacity, out ulong outByteLen, out ulong outError);
}

/// <summary>
/// Default P/Invoke implementation backed by <c>freemvvm_abi_1</c>.
/// </summary>
public sealed class NativeWebViewApi : IWebViewNativeApi
{
    private const string LibraryName = "freemvvm_abi_1";

    public static NativeWebViewApi Instance { get; } = new();

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, ExactSpelling = true, EntryPoint = "ui_abi_1_handle_retain")]
    private static extern uint NativeHandleRetain(ulong handle, out ulong outHandle);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, ExactSpelling = true, EntryPoint = "ui_abi_1_handle_release")]
    private static extern uint NativeHandleRelease(ulong handle);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, ExactSpelling = true, EntryPoint = "ui_abi_1_error_code")]
    private static extern uint NativeErrorCode(ulong errorHandle, out uint outDomain, out uint outCode);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, ExactSpelling = true, EntryPoint = "ui_abi_1_control_create_webview")]
    private static extern uint NativeControlCreateWebView(ulong runtime, out ulong outWebView, out ulong outError);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, ExactSpelling = true, EntryPoint = "ui_abi_1_webview_set_source_utf8")]
    private static extern uint NativeWebViewSetSourceUtf8(ulong webview, uint format, [In] byte[]? utf8Bytes, ulong byteLen, out ulong outError);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, ExactSpelling = true, EntryPoint = "ui_abi_1_webview_set_stylesheet_utf8")]
    private static extern uint NativeWebViewSetStylesheetUtf8(ulong webview, [In] byte[]? utf8Bytes, ulong byteLen, out ulong outError);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, ExactSpelling = true, EntryPoint = "ui_abi_1_webview_set_viewport")]
    private static extern uint NativeWebViewSetViewport(ulong webview, double widthDip, double heightDip, out ulong outError);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, ExactSpelling = true, EntryPoint = "ui_abi_1_webview_bind_source_property")]
    private static extern uint NativeWebViewBindSourceProperty(ulong webview, uint format, ulong utf8Property, out ulong outBinding, out ulong outError);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, ExactSpelling = true, EntryPoint = "ui_abi_1_webview_expose_property_utf8")]
    private static extern uint NativeWebViewExposePropertyUtf8(ulong webview, [In] byte[]? aliasUtf8, ulong aliasLen, ulong property, byte allowWrite, out ulong outError);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, ExactSpelling = true, EntryPoint = "ui_abi_1_webview_expose_command_utf8")]
    private static extern uint NativeWebViewExposeCommandUtf8(
        ulong webview,
        [In] byte[]? aliasUtf8,
        ulong aliasLen,
        UiPostCallback callback,
        IntPtr userdata,
        UiDestroyCallback destroy,
        out ulong outError);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, ExactSpelling = true, EntryPoint = "ui_abi_1_webview_eval_script_utf8")]
    private static extern uint NativeWebViewEvalScriptUtf8(ulong webview, [In] byte[]? scriptUtf8, ulong scriptLen, out ulong outError);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, ExactSpelling = true, EntryPoint = "ui_abi_1_webview_click_at")]
    private static extern uint NativeWebViewClickAt(ulong webview, double xDip, double yDip, out ulong outError);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, ExactSpelling = true, EntryPoint = "ui_abi_1_webview_text_content_utf8")]
    private static extern uint NativeWebViewTextContentUtf8(ulong webview, [Out] byte[]? buffer, ulong capacity, out ulong outByteLen, out ulong outError);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, ExactSpelling = true, EntryPoint = "ui_abi_1_webview_render_rgba")]
    private static extern uint NativeWebViewRenderRgba(ulong webview, uint widthPx, uint heightPx, [Out] byte[]? rgbaBuffer, ulong capacity, out ulong outByteLen, out ulong outError);

    public uint HandleRetain(ulong handle, out ulong outHandle) =>
        NativeHandleRetain(handle, out outHandle);

    public uint HandleRelease(ulong handle) =>
        NativeHandleRelease(handle);

    public uint ErrorCode(ulong errorHandle, out uint outDomain, out uint outCode) =>
        NativeErrorCode(errorHandle, out outDomain, out outCode);

    public uint ControlCreateWebView(ulong runtime, out ulong outWebView, out ulong outError) =>
        NativeControlCreateWebView(runtime, out outWebView, out outError);

    public uint WebViewSetSourceUtf8(ulong webview, uint format, byte[] utf8Bytes, ulong byteLen, out ulong outError) =>
        NativeWebViewSetSourceUtf8(webview, format, byteLen == 0 ? null : utf8Bytes, byteLen, out outError);

    public uint WebViewSetStylesheetUtf8(ulong webview, byte[] utf8Bytes, ulong byteLen, out ulong outError) =>
        NativeWebViewSetStylesheetUtf8(webview, byteLen == 0 ? null : utf8Bytes, byteLen, out outError);

    public uint WebViewSetViewport(ulong webview, double widthDip, double heightDip, out ulong outError) =>
        NativeWebViewSetViewport(webview, widthDip, heightDip, out outError);

    public uint WebViewBindSourceProperty(ulong webview, uint format, ulong utf8Property, out ulong outBinding, out ulong outError) =>
        NativeWebViewBindSourceProperty(webview, format, utf8Property, out outBinding, out outError);

    public uint WebViewExposePropertyUtf8(ulong webview, byte[] aliasUtf8, ulong aliasLen, ulong property, byte allowWrite, out ulong outError) =>
        NativeWebViewExposePropertyUtf8(webview, aliasLen == 0 ? null : aliasUtf8, aliasLen, property, allowWrite, out outError);

    public uint WebViewExposeCommandUtf8(
        ulong webview,
        byte[] aliasUtf8,
        ulong aliasLen,
        UiPostCallback callback,
        IntPtr userdata,
        UiDestroyCallback destroy,
        out ulong outError) =>
        NativeWebViewExposeCommandUtf8(webview, aliasLen == 0 ? null : aliasUtf8, aliasLen, callback, userdata, destroy, out outError);

    public uint WebViewEvalScriptUtf8(ulong webview, byte[] scriptUtf8, ulong scriptLen, out ulong outError) =>
        NativeWebViewEvalScriptUtf8(webview, scriptLen == 0 ? null : scriptUtf8, scriptLen, out outError);

    public uint WebViewClickAt(ulong webview, double xDip, double yDip, out ulong outError) =>
        NativeWebViewClickAt(webview, xDip, yDip, out outError);

    public uint WebViewTextContentUtf8(ulong webview, byte[]? buffer, ulong capacity, out ulong outByteLen, out ulong outError) =>
        NativeWebViewTextContentUtf8(webview, buffer, capacity, out outByteLen, out outError);

    public uint WebViewRenderRgba(ulong webview, uint widthPx, uint heightPx, byte[]? rgbaBuffer, ulong capacity, out ulong outByteLen, out ulong outError) =>
        NativeWebViewRenderRgba(webview, widthPx, heightPx, rgbaBuffer, capacity, out outByteLen, out outError);
}

/// <summary>
/// Owned binding handle returned by <see cref="WebView.BindSourceProperty"/>.
/// </summary>
public sealed class WebViewBinding : IDisposable
{
    private readonly IWebViewNativeApi _api;
    private ulong _token;

    internal WebViewBinding(ulong token, IWebViewNativeApi api)
    {
        _token = token;
        _api = api;
    }

    public bool IsValid => Volatile.Read(ref _token) != 0;

    public Result Close()
    {
        ulong token = Volatile.Read(ref _token);
        if (token == 0)
        {
            return Result.Success();
        }

        uint status = _api.HandleRelease(token);
        if (status == 0)
        {
            Volatile.Write(ref _token, 0);
            return Result.Success();
        }

        return Result.Failure(new UiError(status));
    }

    public void Dispose()
    {
        _ = Close();
    }
}

/// <summary>
/// Safe managed wrapper for the FreeMVVM <c>WebView</c> control (<c>UNIT-WEBVIEW-013</c>).
/// </summary>
public sealed class WebView : IDisposable
{
    public const uint UiOk = 0;
    public const uint UiInvalidArgument = 2;
    public const uint UiClosed = 6;
    public const uint UiBufferTooSmall = 8;
    public const uint UiInternalError = 255;

    private static readonly UiPostCallback s_commandInvokeDelegate = OnCommandInvoke;
    private static readonly UiDestroyCallback s_commandDestroyDelegate = OnCommandDestroy;
    private static readonly UTF8Encoding s_strictUtf8 = new(encoderShouldEmitUTF8Identifier: false, throwOnInvalidBytes: true);

    private readonly IWebViewNativeApi _api;
    private readonly List<ulong> _deferredReaperTokens = new(capacity: 1);
    private ulong _token;
    private int _activeLeases;
    private int _disposing;

    private WebView(ulong token, IWebViewNativeApi api)
    {
        _token = token;
        _api = api;
    }

    public static Result<WebView> Create(ulong runtimeHandle, IWebViewNativeApi? api = null)
    {
        IWebViewNativeApi resolvedApi = api ?? NativeWebViewApi.Instance;
        if (runtimeHandle == 0)
        {
            return Result<WebView>.Failure(new UiError(UiInvalidArgument));
        }

        uint status = resolvedApi.ControlCreateWebView(runtimeHandle, out ulong outWebView, out ulong outError);
        if (status != UiOk)
        {
            return Result<WebView>.Failure(MakeError(resolvedApi, status, outError));
        }

        if (outWebView == 0)
        {
            return Result<WebView>.Failure(new UiError(UiInternalError));
        }

        return Result<WebView>.Success(new WebView(outWebView, resolvedApi));
    }

    public Result SetSource(WebViewFormat format, string source)
    {
        if (source is null)
        {
            return Result.Failure(new UiError(UiInvalidArgument));
        }

        if (!TryEnterLease(out ulong token))
        {
            return Result.Failure(new UiError(UiClosed));
        }

        try
        {
            byte[] bytes = s_strictUtf8.GetBytes(source);
            uint status = _api.WebViewSetSourceUtf8(token, (uint)format, bytes, (ulong)bytes.Length, out ulong outError);
            return status == UiOk
                ? Result.Success()
                : Result.Failure(MakeError(_api, status, outError));
        }
        finally
        {
            ExitLease();
        }
    }

    public Result SetStylesheet(string css)
    {
        if (css is null)
        {
            return Result.Failure(new UiError(UiInvalidArgument));
        }

        if (!TryEnterLease(out ulong token))
        {
            return Result.Failure(new UiError(UiClosed));
        }

        try
        {
            byte[] bytes = s_strictUtf8.GetBytes(css);
            uint status = _api.WebViewSetStylesheetUtf8(token, bytes, (ulong)bytes.Length, out ulong outError);
            return status == UiOk
                ? Result.Success()
                : Result.Failure(MakeError(_api, status, outError));
        }
        finally
        {
            ExitLease();
        }
    }

    public Result SetViewport(double widthDip, double heightDip)
    {
        if (!TryEnterLease(out ulong token))
        {
            return Result.Failure(new UiError(UiClosed));
        }

        try
        {
            uint status = _api.WebViewSetViewport(token, widthDip, heightDip, out ulong outError);
            return status == UiOk
                ? Result.Success()
                : Result.Failure(MakeError(_api, status, outError));
        }
        finally
        {
            ExitLease();
        }
    }

    public Result<WebViewBinding> BindSourceProperty(WebViewFormat format, ulong utf8PropertyHandle)
    {
        if (!TryEnterLease(out ulong token))
        {
            return Result<WebViewBinding>.Failure(new UiError(UiClosed));
        }

        try
        {
            uint status = _api.WebViewBindSourceProperty(token, (uint)format, utf8PropertyHandle, out ulong outBinding, out ulong outError);
            if (status != UiOk)
            {
                return Result<WebViewBinding>.Failure(MakeError(_api, status, outError));
            }

            if (outBinding == 0)
            {
                return Result<WebViewBinding>.Failure(new UiError(UiInternalError));
            }

            return Result<WebViewBinding>.Success(new WebViewBinding(outBinding, _api));
        }
        finally
        {
            ExitLease();
        }
    }

    public Result ExposeProperty(string alias, ulong propertyHandle, bool allowWrite)
    {
        if (alias is null)
        {
            return Result.Failure(new UiError(UiInvalidArgument));
        }

        if (!TryEnterLease(out ulong token))
        {
            return Result.Failure(new UiError(UiClosed));
        }

        try
        {
            byte[] aliasBytes = s_strictUtf8.GetBytes(alias);
            uint status = _api.WebViewExposePropertyUtf8(
                token,
                aliasBytes,
                (ulong)aliasBytes.Length,
                propertyHandle,
                allowWrite ? (byte)1 : (byte)0,
                out ulong outError);
            return status == UiOk
                ? Result.Success()
                : Result.Failure(MakeError(_api, status, outError));
        }
        finally
        {
            ExitLease();
        }
    }

    public Result ExposeCommand(string alias, Action callback)
    {
        if (alias is null || callback is null)
        {
            return Result.Failure(new UiError(UiInvalidArgument));
        }

        if (!TryEnterLease(out ulong token))
        {
            return Result.Failure(new UiError(UiClosed));
        }

        try
        {
            byte[] aliasBytes = s_strictUtf8.GetBytes(alias);
            GCHandle gcHandle = GCHandle.Alloc(callback, GCHandleType.Normal);
            IntPtr userdata = GCHandle.ToIntPtr(gcHandle);
            uint status = _api.WebViewExposeCommandUtf8(
                token,
                aliasBytes,
                (ulong)aliasBytes.Length,
                s_commandInvokeDelegate,
                userdata,
                s_commandDestroyDelegate,
                out ulong outError);
            if (status != UiOk)
            {
                gcHandle.Free();
                return Result.Failure(MakeError(_api, status, outError));
            }

            return Result.Success();
        }
        finally
        {
            ExitLease();
        }
    }

    public Result EvalScript(string script)
    {
        if (script is null)
        {
            return Result.Failure(new UiError(UiInvalidArgument));
        }

        if (!TryEnterLease(out ulong token))
        {
            return Result.Failure(new UiError(UiClosed));
        }

        try
        {
            byte[] bytes = s_strictUtf8.GetBytes(script);
            uint status = _api.WebViewEvalScriptUtf8(token, bytes, (ulong)bytes.Length, out ulong outError);
            return status == UiOk
                ? Result.Success()
                : Result.Failure(MakeError(_api, status, outError));
        }
        finally
        {
            ExitLease();
        }
    }

    public Result ClickAt(double xDip, double yDip)
    {
        if (!TryEnterLease(out ulong token))
        {
            return Result.Failure(new UiError(UiClosed));
        }

        try
        {
            uint status = _api.WebViewClickAt(token, xDip, yDip, out ulong outError);
            return status == UiOk
                ? Result.Success()
                : Result.Failure(MakeError(_api, status, outError));
        }
        finally
        {
            ExitLease();
        }
    }

    public Result<string> TextContent()
    {
        if (!TryEnterLease(out ulong token))
        {
            return Result<string>.Failure(new UiError(UiClosed));
        }

        try
        {
            uint status = _api.WebViewTextContentUtf8(token, null, 0, out ulong required, out ulong outError);
            if (status != UiOk)
            {
                return Result<string>.Failure(MakeError(_api, status, outError));
            }

            if (required == 0)
            {
                return Result<string>.Success(string.Empty);
            }

            if (required > int.MaxValue)
            {
                return Result<string>.Failure(new UiError(UiInvalidArgument));
            }

            byte[] buffer = new byte[(int)required];
            uint copyStatus = _api.WebViewTextContentUtf8(token, buffer, required, out ulong written, out outError);
            if (copyStatus != UiOk)
            {
                return Result<string>.Failure(MakeError(_api, copyStatus, outError));
            }

            if (written > (ulong)buffer.Length)
            {
                return Result<string>.Failure(new UiError(UiBufferTooSmall));
            }

            string decoded = s_strictUtf8.GetString(buffer, 0, (int)written);
            return Result<string>.Success(decoded);
        }
        finally
        {
            ExitLease();
        }
    }

    public Result<byte[]> RenderRgba(uint widthPx, uint heightPx)
    {
        if (!TryEnterLease(out ulong token))
        {
            return Result<byte[]>.Failure(new UiError(UiClosed));
        }

        try
        {
            uint status = _api.WebViewRenderRgba(token, widthPx, heightPx, null, 0, out ulong required, out ulong outError);
            if (status != UiOk)
            {
                return Result<byte[]>.Failure(MakeError(_api, status, outError));
            }

            if (required > int.MaxValue)
            {
                return Result<byte[]>.Failure(new UiError(UiInvalidArgument));
            }

            byte[] rgba = new byte[(int)required];
            uint copyStatus = _api.WebViewRenderRgba(token, widthPx, heightPx, rgba, required, out ulong written, out outError);
            if (copyStatus != UiOk)
            {
                return Result<byte[]>.Failure(MakeError(_api, copyStatus, outError));
            }

            if (written != (ulong)rgba.Length)
            {
                return Result<byte[]>.Failure(new UiError(UiBufferTooSmall));
            }

            return Result<byte[]>.Success(rgba);
        }
        finally
        {
            ExitLease();
        }
    }

    public Result Close()
    {
        Interlocked.Exchange(ref _disposing, 1);
        if (Volatile.Read(ref _activeLeases) != 0)
        {
            return Result.Success();
        }

        return TryReleaseUnderlyingToken();
    }

    public void Dispose()
    {
        _ = Close();
    }

    private bool TryEnterLease(out ulong token)
    {
        Interlocked.Increment(ref _activeLeases);
        if (Volatile.Read(ref _disposing) != 0)
        {
            ExitLease();
            token = 0;
            return false;
        }

        token = Volatile.Read(ref _token);
        if (token == 0)
        {
            ExitLease();
            return false;
        }

        return true;
    }

    private void ExitLease()
    {
        if (Interlocked.Decrement(ref _activeLeases) == 0 && Volatile.Read(ref _disposing) != 0)
        {
            _ = TryReleaseUnderlyingToken();
        }
    }

    private Result TryReleaseUnderlyingToken()
    {
        ulong token = Interlocked.Exchange(ref _token, 0);
        if (token == 0)
        {
            return Result.Success();
        }

        uint status = _api.HandleRelease(token);
        if (status == UiOk)
        {
            return Result.Success();
        }

        lock (_deferredReaperTokens)
        {
            _deferredReaperTokens.Add(token);
        }

        return Result.Failure(new UiError(status));
    }

    private static UiError MakeError(IWebViewNativeApi api, uint status, ulong errorHandle)
    {
        if (errorHandle == 0)
        {
            return new UiError(status);
        }

        uint codeStatus = api.ErrorCode(errorHandle, out uint domain, out uint code);
        _ = api.HandleRelease(errorHandle);
        return codeStatus == UiOk
            ? new UiError(status, domain, code)
            : new UiError(status);
    }

    private static void OnCommandInvoke(IntPtr userdata)
    {
        if (userdata == IntPtr.Zero)
        {
            return;
        }

        try
        {
            GCHandle handle = GCHandle.FromIntPtr(userdata);
            if (handle.Target is Action callback)
            {
                callback();
            }
        }
        catch
        {
            // Managed callback exceptions must never escape across the C ABI boundary.
        }
    }

    private static void OnCommandDestroy(IntPtr userdata)
    {
        if (userdata == IntPtr.Zero)
        {
            return;
        }

        try
        {
            GCHandle handle = GCHandle.FromIntPtr(userdata);
            if (handle.IsAllocated)
            {
                handle.Free();
            }
        }
        catch
        {
            // Destroy callbacks must never throw across the C ABI boundary.
        }
    }
}
