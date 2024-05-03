// <auto-generated/> by Diplomat

#pragma warning disable 0105
using System;
using System.Runtime.InteropServices;

using DiplomatFeatures.Diplomat;
#pragma warning restore 0105

namespace DiplomatFeatures.Raw;

#nullable enable

[StructLayout(LayoutKind.Sequential)]
public partial struct Utf16Wrap
{
    private const string NativeLib = "diplomat_feature_tests";

    [DllImport(NativeLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "Utf16Wrap_borrow_cont", ExactSpelling = true)]
    public static unsafe extern ushort[] BorrowCont(Utf16Wrap* self);

    [DllImport(NativeLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "Utf16Wrap_owned", ExactSpelling = true)]
    public static unsafe extern ushort[] Owned(Utf16Wrap* self);

    [DllImport(NativeLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "Utf16Wrap_destroy", ExactSpelling = true)]
    public static unsafe extern void Destroy(Utf16Wrap* self);
}