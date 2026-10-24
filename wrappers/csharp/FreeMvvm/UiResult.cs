using System;
using System.Text;
using System.Threading;

namespace FreeMvvm;

public sealed class UiErrorSnapshot : IDisposable
{
    private readonly byte[]? _messageUtf8;
    private TokenOwner? _nativeOwner;

    internal UiErrorSnapshot(
        uint rawStatus,
        uint? domain = null,
        uint? code = null,
        byte[]? messageUtf8 = null,
        TokenOwner? nativeOwner = null)
    {
        RawStatus = rawStatus;
        Domain = domain;
        Code = code;
        _messageUtf8 = messageUtf8 is null ? null : (byte[])messageUtf8.Clone();
        _nativeOwner = nativeOwner;
    }

    public uint RawStatus { get; }

    public uint? Domain { get; }

    public uint? Code { get; }

    public byte[]? MessageUtf8 => _messageUtf8 is null ? null : (byte[])_messageUtf8.Clone();

    public string? Message
    {
        get
        {
            if (_messageUtf8 is null)
            {
                return null;
            }

            try
            {
                return new UTF8Encoding(false, true).GetString(_messageUtf8);
            }
            catch (DecoderFallbackException)
            {
                return null;
            }
        }
    }

    public void Dispose() => Interlocked.Exchange(ref _nativeOwner, null)?.Dispose();
}

public sealed class UiException : Exception
{
    public UiException(UiErrorSnapshot error)
        : base(error.Message ?? $"FreeMvvm operation failed with status {error.RawStatus}.")
    {
        Error = error;
    }

    public UiErrorSnapshot Error { get; }
}

public readonly struct UiResult<T>
{
    private readonly T? _value;

    private UiResult(bool isSuccess, T? value, UiErrorSnapshot? error)
    {
        IsSuccess = isSuccess;
        _value = value;
        Error = error;
    }

    public bool IsSuccess { get; }

    public bool IsFailure => !IsSuccess;

    public UiErrorSnapshot? Error { get; }

    public T Value => IsSuccess
        ? _value!
        : throw new UiException(Error ?? new UiErrorSnapshot(0xFFFF_FFFF));

    public T GetValueOrThrow() => Value;

    internal static UiResult<T> Success(T value) => new(true, value, null);

    internal static UiResult<T> Failure(UiErrorSnapshot error) => new(false, default, error);
}
