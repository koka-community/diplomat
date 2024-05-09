//! This module contains functions for formatting types

use crate::c2::CFormatter;
use diplomat_core::ast::{DocsUrlGenerator, MarkdownStyle};
use diplomat_core::hir::{self, TypeContext, TypeId};
use heck::{ToLowerCamelCase, ToSnekCase, ToUpperCamelCase};
use std::borrow::Cow;

/// This type mediates all formatting
///
/// All identifiers from the HIR should go through here before being formatted
/// into the output: This makes it easy to handle reserved words or add rename support
///
/// If you find yourself needing an identifier formatted in a context not yet available here, please add a new method
///
/// This type may be used by other backends attempting to figure out the names
/// of C types and methods.
pub(super) struct KokaFormatter<'tcx> {
    c: CFormatter<'tcx>,
    docs_url_generator: &'tcx DocsUrlGenerator,
    strip_prefix: Option<String>,
}

const INVALID_METHOD_NAMES: &[&str] = &["new", "static", "default"];
const INVALID_FIELD_NAMES: &[&str] = &["new", "static", "default"];
const DISALLOWED_CORE_TYPES: &[&str] = &["Object", "String"];

impl<'tcx> KokaFormatter<'tcx> {
    pub fn new(
        tcx: &'tcx TypeContext,
        docs_url_generator: &'tcx DocsUrlGenerator,
        strip_prefix: Option<String>,
    ) -> Self {
        Self {
            c: CFormatter::new(tcx),
            docs_url_generator,
            strip_prefix,
        }
    }

    pub fn fmt_lifetime_edge_array(
        &self,
        lifetime: hir::Lifetime,
        lifetime_env: &hir::LifetimeEnv,
    ) -> Cow<'static, str> {
        format!("{}Edges", lifetime_env.fmt_lifetime(lifetime)).into()
    }

    pub fn fmt_file_name(&self, name: &str) -> String {
        format!("{name}.kk")
    }

    pub fn fmt_import(&self, path: &str, as_show_hide: Option<&str>) -> Cow<'static, str> {
        format!(
            "import {path}{}{};",
            if as_show_hide.is_some() { " " } else { "" },
            if let Some(s) = as_show_hide { s } else { "" },
        )
        .into()
    }

    pub fn fmt_docs(&self, docs: &hir::Docs) -> String {
        docs.to_markdown(self.docs_url_generator, MarkdownStyle::Normal)
            .trim()
            .replace('\n', "\n// ")
            .replace(" \n", "\n")
            .replace(
                &format!("`{}", self.strip_prefix.as_deref().unwrap_or("")),
                "`",
            )
    }

    pub fn fmt_destructor_name(&self, id: TypeId) -> String {
        self.c.fmt_dtor_name(id)
    }

    /// Resolve and format a named type for use in code
    pub fn fmt_type_name(&self, id: TypeId) -> Cow<'tcx, str> {
        let resolved = self.c.tcx().resolve_type(id);

        let candidate: Cow<str> = if let Some(strip_prefix) = self.strip_prefix.as_ref() {
            resolved
                .name()
                .as_str()
                .strip_prefix(strip_prefix)
                .unwrap_or(resolved.name().as_str())
                .into()
        } else {
            resolved.name().as_str().into()
        };

        if DISALLOWED_CORE_TYPES.contains(&&*candidate) {
            panic!("{candidate:?} is not a valid Koka type name. Please rename.");
        }

        resolved.attrs().rename.apply(candidate)
    }

    /// Resolve and format a named type for use in diagnostics
    /// (don't apply rename rules and such)
    pub fn fmt_type_name_diagnostics(&self, id: TypeId) -> Cow<'tcx, str> {
        self.c.fmt_type_name_diagnostics(id)
    }

    /// Format an enum variant.
    pub fn fmt_enum_variant(&self, variant: &'tcx hir::EnumVariant) -> Cow<'tcx, str> {
        let name = variant.name.as_str().to_upper_camel_case().into();
        variant.attrs.rename.apply(name)
    }

    /// Format a field name or parameter name
    // might need splitting in the future if we decide to support renames here
    pub fn fmt_param_name<'a>(&self, ident: &'a str) -> Cow<'a, str> {
        ident.to_lowercase().to_snek_case().into()
    }

    pub fn fmt_nullable(&self, ident: &str) -> String {
        format!("{ident}?")
    }

    /// Format a method
    pub fn fmt_method_name(&self, method: &hir::Method) -> String {
        // TODO(#60): handle other keywords
        let name = method
            .attrs
            .rename
            .apply(method.name.as_str().into())
            .to_snek_case();
        if INVALID_METHOD_NAMES.contains(&&*name) {
            format!("{name}_")
        } else {
            name
        }
    }

    fn uppercase_first_letter(&self, s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }
    pub fn fmt_constructor_name(&self, name: &Option<String>, method: &hir::Method) -> String {
        let name = self.uppercase_first_letter(
            method
                .attrs
                .rename
                .apply(name.as_deref().unwrap_or(method.name.as_str()).into())
                .to_snek_case()
                .as_str(),
        );

        if INVALID_METHOD_NAMES.contains(&name.as_str()) {
            format!("{name}_")
        } else {
            name
        }
    }

    pub fn fmt_accessor_name(&self, name: &Option<String>, method: &hir::Method) -> String {
        let name = method
            .attrs
            .rename
            .apply(name.as_deref().unwrap_or(method.name.as_str()).into())
            .to_snek_case();

        if INVALID_FIELD_NAMES.contains(&name.as_str()) {
            format!("{name}_")
        } else {
            name
        }
    }

    pub fn fmt_c_method_name<'a>(&self, ty: TypeId, method: &'a hir::Method) -> Cow<'a, str> {
        self.c.fmt_method_name(ty, method).into()
    }

    pub fn fmt_string(&self) -> &'static str {
        "string"
    }

    pub fn fmt_utf8_primitive(&self) -> &'static str {
        "int8"
    }

    pub fn fmt_utf16_primitive(&self) -> &'static str {
        "int16"
    }

    pub fn fmt_void(&self) -> &'static str {
        "()"
    }

    pub fn fmt_ffi_void(&self) -> &'static str {
        "()"
    }

    pub fn fmt_pointer(&self, target: &str) -> String {
        format!("c-pointer<{target}>")
    }

    pub fn fmt_usize(&self, cast: bool) -> &'static str {
        self.fmt_primitive_as_ffi(hir::PrimitiveType::IntSize(hir::IntSizeType::Usize), cast)
    }

    pub fn fmt_type_as_ident(&self, ty: Option<&str>) -> String {
        ty.unwrap_or("()").replace('-', "")
    }

    pub fn fmt_enum_as_ffi(&self, cast: bool) -> &'static str {
        self.fmt_primitive_as_ffi(hir::PrimitiveType::Int(hir::IntType::I32), cast)
    }

    pub fn fmt_primitive_as_ffi(&self, prim: hir::PrimitiveType, cast: bool) -> &'static str {
        use diplomat_core::hir::{FloatType, IntSizeType, IntType, PrimitiveType};
        if cast {
            match prim {
                PrimitiveType::Bool => "bool",
                PrimitiveType::Char => "char",
                PrimitiveType::Int(_) | PrimitiveType::IntSize(_) => "int",
                PrimitiveType::Byte => "int8",
                PrimitiveType::Float(_) => "float64",
                PrimitiveType::Int128(_) => panic!("i128 not supported in Dart"),
            }
        } else {
            match prim {
                PrimitiveType::Bool => "bool",
                PrimitiveType::Char => "char",
                PrimitiveType::Int(IntType::I8) => "int8",
                PrimitiveType::Int(IntType::U8) | PrimitiveType::Byte => "int8",
                PrimitiveType::Int(IntType::I16) => "int16",
                PrimitiveType::Int(IntType::U16) => "int16",
                PrimitiveType::Int(IntType::I32) => "int32",
                PrimitiveType::Int(IntType::U32) => "int32",
                PrimitiveType::Int(IntType::I64) => "int64",
                PrimitiveType::Int(IntType::U64) => "int64",
                PrimitiveType::IntSize(IntSizeType::Isize) => "intptr_t",
                PrimitiveType::IntSize(IntSizeType::Usize) => "ssize_t",
                PrimitiveType::Float(FloatType::F32) => "float32",
                PrimitiveType::Float(FloatType::F64) => "float64",
                PrimitiveType::Int128(_) => panic!("i128 not supported in Dart"),
            }
        }
    }

    pub fn fmt_primitive_list_type(&self, prim: hir::PrimitiveType) -> &'static str {
        use diplomat_core::hir::PrimitiveType;
        match prim {
            PrimitiveType::Bool => "list<bool>",
            PrimitiveType::Char => "list<char>",
            PrimitiveType::Byte => "bytes",
            PrimitiveType::Int(_) | PrimitiveType::IntSize(_) => "list<int>",
            PrimitiveType::Float(_) => "list<float64>",
            PrimitiveType::Int128(_) => panic!("i128 not supported in Dart"),
        }
    }

    pub fn fmt_primitive_list_view(&self, prim: hir::PrimitiveType) -> &'static str {
        use diplomat_core::hir::{FloatType, IntSizeType, IntType, PrimitiveType};
        match prim {
            PrimitiveType::Bool => ".boolView",
            PrimitiveType::Char => ".uint32View",
            PrimitiveType::Byte => "",
            PrimitiveType::Int(IntType::I8) => ".int8View",
            PrimitiveType::Int(IntType::U8) => ".uint8View",
            PrimitiveType::Int(IntType::I16) => ".int16View",
            PrimitiveType::Int(IntType::U16) => ".uint16View",
            PrimitiveType::Int(IntType::I32) => ".int32View",
            PrimitiveType::Int(IntType::U32) => ".uint32View",
            PrimitiveType::Int(IntType::I64) => ".int64View",
            PrimitiveType::Int(IntType::U64) => ".uint64View",
            PrimitiveType::IntSize(IntSizeType::Usize) => ".usizeView",
            PrimitiveType::IntSize(IntSizeType::Isize) => ".isizeView",
            PrimitiveType::Float(FloatType::F32) => ".float32View",
            PrimitiveType::Float(FloatType::F64) => ".float64View",
            PrimitiveType::Int128(_) => panic!("i128 not supported in Dart"),
        }
    }

    pub fn fmt_slice_type(&self, prim: hir::PrimitiveType) -> &'static str {
        use diplomat_core::hir::{FloatType, IntSizeType, IntType, PrimitiveType};
        match prim {
            PrimitiveType::Bool => "_SliceBool",
            PrimitiveType::Char => "_SliceRune",
            PrimitiveType::Int(IntType::I8) => "_SliceInt8",
            PrimitiveType::Int(IntType::U8) | PrimitiveType::Byte => "_SliceUint8",
            PrimitiveType::Int(IntType::I16) => "_SliceInt16",
            PrimitiveType::Int(IntType::U16) => "_SliceUint16",
            PrimitiveType::Int(IntType::I32) => "_SliceInt32",
            PrimitiveType::Int(IntType::U32) => "_SliceUint32",
            PrimitiveType::Int(IntType::I64) => "_SliceInt64",
            PrimitiveType::Int(IntType::U64) => "_SliceUint64",
            PrimitiveType::IntSize(IntSizeType::Usize) => "_SliceUsize",
            PrimitiveType::IntSize(IntSizeType::Isize) => "_SliceIsize",
            PrimitiveType::Float(FloatType::F32) => "_SliceFloat",
            PrimitiveType::Float(FloatType::F64) => "_SliceDouble",
            PrimitiveType::Int128(_) => panic!("i128 not supported in Dart"),
        }
    }

    pub fn fmt_utf8_slice_type(&self) -> &'static str {
        "_SliceUtf8"
    }

    pub fn fmt_utf16_slice_type(&self) -> &'static str {
        "_SliceUtf16"
    }
}
