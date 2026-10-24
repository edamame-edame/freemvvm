using System;

namespace FreeMvvm;

internal sealed class PreparedError
{
    private readonly TokenOwner _owner;

    internal PreparedError(INativeApi api) => _owner = new TokenOwner(api);

    internal UiErrorSnapshot Capture(uint operationStatus, ulong errorToken)
    {
        if (errorToken == 0)
        {
            return new UiErrorSnapshot(operationStatus);
        }

        try
        {
            _owner.Adopt(errorToken);
        }
        catch
        {
            return new UiErrorSnapshot(operationStatus);
        }

        uint? errorStatus = null;
        uint? domain = null;
        uint? code = null;
        byte[]? message = null;
        UiResult<CallLease> leaseResult = _owner.AcquireLease();
        if (leaseResult.IsSuccess)
        {
            using CallLease lease = leaseResult.Value;
            try
            {
                uint status = _owner.Api.ErrorStatus(lease.Token, out uint nativeErrorStatus);
                if (status == UiStatus.Ok)
                {
                    errorStatus = nativeErrorStatus;
                }

                status = _owner.Api.ErrorCode(lease.Token, out uint nativeDomain, out uint nativeCode);
                if (status == UiStatus.Ok)
                {
                    domain = nativeDomain;
                    code = nativeCode;
                }

                message = ReadMessage(lease.Token);
            }
            catch
            {
                // The operation status remains authoritative if detail inspection fails.
            }
        }

        _owner.Dispose();
        return new UiErrorSnapshot(errorStatus ?? operationStatus, domain, code, message);
    }

    internal UiErrorSnapshot CaptureOwnedSnapshot(ulong errorToken)
    {
        if (errorToken == 0)
        {
            return new UiErrorSnapshot(UiStatus.InternalError);
        }

        try
        {
            _owner.Adopt(errorToken);
        }
        catch
        {
            try
            {
                _owner.Api.HandleRelease(errorToken);
            }
            catch
            {
                // Preserve the public boundary even if a malformed token cannot be inspected.
            }

            return new UiErrorSnapshot(UiStatus.InternalError);
        }

        uint? errorStatus = null;
        uint? domain = null;
        uint? code = null;
        byte[]? message = null;
        UiResult<CallLease> leaseResult = _owner.AcquireLease();
        if (leaseResult.IsSuccess)
        {
            using CallLease lease = leaseResult.Value;
            try
            {
                uint status = _owner.Api.ErrorStatus(lease.Token, out uint nativeErrorStatus);
                if (status == UiStatus.Ok)
                {
                    errorStatus = nativeErrorStatus;
                }

                status = _owner.Api.ErrorCode(lease.Token, out uint nativeDomain, out uint nativeCode);
                if (status == UiStatus.Ok)
                {
                    domain = nativeDomain;
                    code = nativeCode;
                }

                message = ReadMessage(lease.Token);
            }
            catch
            {
                // The owned snapshot remains available even when detail inspection fails.
            }
        }

        return new UiErrorSnapshot(errorStatus ?? UiStatus.InternalError, domain, code, message, _owner);
    }

    private byte[]? ReadMessage(ulong error)
    {
        unsafe
        {
            uint status = _owner.Api.ErrorMessageUtf8(error, null, 0, out ulong length);
            if (status != UiStatus.Ok || length == 0 || length > int.MaxValue)
            {
                return null;
            }

            byte[] bytes = new byte[(int)length];
            fixed (byte* buffer = bytes)
            {
                status = _owner.Api.ErrorMessageUtf8(error, buffer, length, out ulong copied);
                if (status != UiStatus.Ok || copied != length)
                {
                    return null;
                }
            }

            return bytes;
        }
    }
}
