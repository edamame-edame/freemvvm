using System;

namespace FreeMvvm;

/// <summary>A canonical signed decimal value with a 58-bit coefficient and scale from 0 through 31.</summary>
public readonly struct DecimalValue
{
    private const ulong MaxCoefficient = (1UL << 58) - 1;
    private const ulong CoefficientMask = MaxCoefficient;
    private const int MaxSystemDecimalScale = 28;

    private readonly ulong _bits;

    private DecimalValue(ulong bits) => _bits = bits;

    public bool IsNegative => (_bits >> 63) != 0;

    public byte Scale => (byte)((_bits >> 58) & 0x1F);

    public ulong Coefficient => _bits & CoefficientMask;

    internal ulong Bits => _bits;

    public static UiResult<DecimalValue> TryFromParts(bool negative, byte scale, ulong coefficient)
    {
        if (scale > 31 || coefficient > MaxCoefficient)
        {
            return UiResult<DecimalValue>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        ulong bits = (negative ? 1UL << 63 : 0) | ((ulong)scale << 58) | coefficient;
        return UiResult<DecimalValue>.Success(new DecimalValue(bits));
    }

    public static UiResult<DecimalValue> TryFromDecimal(decimal value)
    {
        int[] parts = decimal.GetBits(value);
        UInt128 coefficient = (uint)parts[0]
            | ((UInt128)(uint)parts[1] << 32)
            | ((UInt128)(uint)parts[2] << 64);
        uint flags = (uint)parts[3];
        byte scale = (byte)((flags >> 16) & 0xFF);

        while (coefficient > MaxCoefficient && scale > 0 && coefficient % 10 == 0)
        {
            coefficient /= 10;
            scale--;
        }

        if (coefficient > MaxCoefficient)
        {
            return UiResult<DecimalValue>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        ulong bits = ((flags & 0x8000_0000) != 0 ? 1UL << 63 : 0)
            | ((ulong)scale << 58)
            | (ulong)coefficient;
        return UiResult<DecimalValue>.Success(new DecimalValue(bits));
    }

    public UiResult<decimal> TryToDecimal()
    {
        ulong coefficient = Coefficient;
        byte scale = Scale;
        while (scale > MaxSystemDecimalScale && coefficient % 10 == 0)
        {
            coefficient /= 10;
            scale--;
        }

        if (scale > MaxSystemDecimalScale)
        {
            return UiResult<decimal>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        int low = unchecked((int)(uint)coefficient);
        int middle = unchecked((int)(uint)(coefficient >> 32));
        return UiResult<decimal>.Success(new decimal(low, middle, 0, IsNegative, scale));
    }
}
