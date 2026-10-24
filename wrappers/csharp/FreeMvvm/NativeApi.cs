using System;
using System.IO;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace FreeMvvm;

internal interface INativeApi
{
    uint RuntimeCreate(out ulong runtime, out ulong error);

    uint RuntimeIsUiThread(ulong runtime, out byte isUiThread);

    uint PropertyCreateBool(ulong runtime, byte initial, out ulong property, out ulong error);

    uint PropertyCreateI64(ulong runtime, long initial, out ulong property, out ulong error);

    uint PropertyCreateF64(ulong runtime, double initial, out ulong property, out ulong error);

    uint PropertyCreateUtf8(ulong runtime, byte[] initial, out ulong property, out ulong error);

    uint PropertyGetBool(ulong property, out byte value);

    uint PropertyGetI64(ulong property, out long value);

    uint PropertyGetF64(ulong property, out double value);

    uint PropertyGetUtf8(ulong property, byte[] buffer, ulong capacity, out ulong length);

    uint PropertySetBool(ulong property, byte value, out ulong error);

    uint PropertySetI64(ulong property, long value, out ulong error);

    uint PropertySetF64(ulong property, double value, out ulong error);

    uint PropertySetUtf8(ulong property, byte[] value, out ulong error);

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

    public uint PropertyCreateBool(ulong runtime, byte initial, out ulong property, out ulong error) =>
        NativeMethods.PropertyCreateBool(runtime, initial, out property, out error);

    public uint PropertyCreateI64(ulong runtime, long initial, out ulong property, out ulong error) =>
        NativeMethods.PropertyCreateI64(runtime, initial, out property, out error);

    public uint PropertyCreateF64(ulong runtime, double initial, out ulong property, out ulong error) =>
        NativeMethods.PropertyCreateF64(runtime, initial, out property, out error);

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

    public unsafe uint PropertySetUtf8(ulong property, byte[] value, out ulong error)
    {
        fixed (byte* bytes = value)
        {
            return NativeMethods.PropertySetUtf8(property, bytes, (ulong)value.Length, out error);
        }
    }

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

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_property_create_bool")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint PropertyCreateBool(ulong runtime, byte initial, out ulong property, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_property_create_i64")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint PropertyCreateI64(ulong runtime, long initial, out ulong property, out ulong error);

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_property_create_f64")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint PropertyCreateF64(ulong runtime, double initial, out ulong property, out ulong error);

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

    [LibraryImport(LibraryName, EntryPoint = "ui_abi_1_property_set_utf8")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial uint PropertySetUtf8(ulong property, byte* value, ulong byteLength, out ulong error);

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
