use heck::ShoutySnakeCase;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use witx::*;

pub fn generate(wasi: &Path) -> String {
    let doc = witx::load(&[wasi.join("phases/snapshot/witx/wasi_snapshot_preview1.witx")]).unwrap();

    let mut raw = String::new();
    raw.push_str(
        "\
// This file is automatically generated, DO NOT EDIT
//
// To regenerate this file run the `crates/generate-raw` command

#![allow(non_camel_case_types)]

",
    );
    for ty in doc.datatypes() {
        ty.render(&mut raw);
        raw.push_str("\n");
    }
    for m in doc.modules() {
        m.render(&mut raw);
        raw.push_str("\n");
    }

    let mut rustfmt = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    rustfmt
        .stdin
        .take()
        .unwrap()
        .write_all(raw.as_bytes())
        .unwrap();
    let mut ret = String::new();
    rustfmt
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut ret)
        .unwrap();
    let status = rustfmt.wait().unwrap();
    assert!(status.success());
    return ret;
}

trait Render {
    fn render(&self, src: &mut String);
}

impl Render for Datatype {
    fn render(&self, src: &mut String) {
        match &self.variant {
            witx::DatatypeVariant::Alias(a) => a.render(src),
            witx::DatatypeVariant::Enum(e) => e.render(src),
            witx::DatatypeVariant::Flags(f) => f.render(src),
            witx::DatatypeVariant::Struct(s) => s.render(src),
            witx::DatatypeVariant::Union(s) => s.render(src),
            witx::DatatypeVariant::Handle(s) => s.render(src),
        }
    }
}

impl Render for UnionDatatype {
    fn render(&self, src: &mut String) {
        src.push_str("#[repr(C)]\n");
        src.push_str("#[derive(Copy, Clone)]\n");
        src.push_str(&format!("pub union __wasi_{}_t {{\n", self.name.as_str()));
        for variant in self.variants.iter() {
            src.push_str("pub ");
            variant.name.render(src);
            src.push_str(": ");
            variant.type_.render(src);
            src.push_str(",\n");
        }
        src.push_str("}");
    }
}

impl Render for StructDatatype {
    fn render(&self, src: &mut String) {
        src.push_str("#[repr(C)]\n");
        src.push_str("#[derive(Copy, Clone)]\n");
        src.push_str(&format!("pub struct __wasi_{}_t {{\n", self.name.as_str()));
        for member in self.members.iter() {
            src.push_str("pub ");
            member.name.render(src);
            src.push_str(": ");
            member.type_.render(src);
            src.push_str(",\n");
        }
        src.push_str("}");
    }
}

impl Render for FlagsDatatype {
    fn render(&self, src: &mut String) {
        src.push_str(&format!("pub type __wasi_{}_t = ", self.name.as_str()));
        self.repr.render(src);
        src.push_str(";\n");
        for (i, variant) in self.flags.iter().enumerate() {
            src.push_str(&format!(
                "pub const __WASI_{}_{}: __wasi_{}_t = 0x{:x};",
                self.name.as_str().to_shouty_snake_case(),
                variant.name.as_str().to_shouty_snake_case(),
                self.name.as_str(),
                1 << i
            ));
        }
    }
}

impl Render for EnumDatatype {
    fn render(&self, src: &mut String) {
        src.push_str(&format!("pub type __wasi_{}_t = ", self.name.as_str()));
        self.repr.render(src);
        src.push_str(";\n");
        for (i, variant) in self.variants.iter().enumerate() {
            src.push_str(&format!(
                "pub const __WASI_{}_{}: __wasi_{}_t = {};",
                self.name.as_str().to_shouty_snake_case(),
                variant.name.as_str().to_shouty_snake_case(),
                self.name.as_str(),
                i
            ));
        }
    }
}

impl Render for IntRepr {
    fn render(&self, src: &mut String) {
        match self {
            IntRepr::U8 => src.push_str("u8"),
            IntRepr::U16 => src.push_str("u16"),
            IntRepr::U32 => src.push_str("u32"),
            IntRepr::U64 => src.push_str("u64"),
        }
    }
}

impl Render for AliasDatatype {
    fn render(&self, src: &mut String) {
        if self.to.passed_by() == DatatypePassedBy::PointerLengthPair {
            return;
        }
        src.push_str(&format!("pub type __wasi_{}_t = ", self.name.as_str()));

        // Give `size` special treatment to translate it to `usize` in Rust
        // instead of `u32`, makes things a bit nicer in Rust.
        if self.name.as_str() == "size" {
            src.push_str("usize");
        } else {
            self.to.render(src);
        }
        src.push(';');
    }
}

impl Render for DatatypeIdent {
    fn render(&self, src: &mut String) {
        match self {
            DatatypeIdent::Builtin(t) => t.render(src),
            DatatypeIdent::Array(_) => unreachable!(),
            DatatypeIdent::Pointer(t) => {
                src.push_str("*mut ");
                t.render(src);
            }
            DatatypeIdent::ConstPointer(t) => {
                src.push_str("*const ");
                t.render(src);
            }
            DatatypeIdent::Ident(t) => {
                src.push_str("__wasi_");
                src.push_str(t.name.as_str());
                src.push_str("_t");
            }
        }
    }
}

impl Render for HandleDatatype {
    fn render(&self, src: &mut String) {
        src.push_str(&format!("pub type __wasi_{}_t = u32;", self.name.as_str()));
    }
}

impl Render for BuiltinType {
    fn render(&self, src: &mut String) {
        match self {
            BuiltinType::String => src.push_str("str"),
            BuiltinType::U8 => src.push_str("u8"),
            BuiltinType::U16 => src.push_str("u16"),
            BuiltinType::U32 => src.push_str("u32"),
            BuiltinType::U64 => src.push_str("u64"),
            BuiltinType::S8 => src.push_str("i8"),
            BuiltinType::S16 => src.push_str("i16"),
            BuiltinType::S32 => src.push_str("i32"),
            BuiltinType::S64 => src.push_str("i64"),
            BuiltinType::F32 => src.push_str("f32"),
            BuiltinType::F64 => src.push_str("f64"),
        }
    }
}

impl Render for Module {
    fn render(&self, src: &mut String) {
        src.push_str("#[link(wasm_import_module =\"");
        src.push_str(self.name.as_str());
        src.push_str("\")]\n");
        src.push_str("extern \"C\" {\n");
        for f in self.funcs() {
            f.render(src);
            src.push_str("\n");
        }
        src.push_str("}");
    }
}

impl Render for InterfaceFunc {
    fn render(&self, src: &mut String) {
        src.push_str("#[link_name = \"");
        src.push_str(self.name.as_str());
        src.push_str("\"]\n");
        src.push_str("pub fn __wasi_");
        src.push_str(self.name.as_str());
        src.push_str("(");
        for param in self.params.iter() {
            param.render(src);
            src.push_str(",");
        }
        for result in self.results.iter().skip(1) {
            result.name.render(src);
            src.push_str(": *mut ");
            result.type_.render(src);
            src.push_str(",");
        }
        src.push_str(")");
        if let Some(result) = self.results.get(0) {
            src.push_str(" -> ");
            result.render(src);
        // special-case the `proc_exit` function for now to be "noreturn", and
        // eventually we'll have an attribute in `*.witx` to specify this as
        // well.
        } else if self.name.as_str() == "proc_exit" {
            src.push_str(" -> !");
        }
        src.push_str(";");
    }
}

impl Render for InterfaceFuncParam {
    fn render(&self, src: &mut String) {
        let is_param = match self.position {
            InterfaceFuncParamPosition::Param(_) => true,
            _ => false,
        };
        match self.type_.passed_by() {
            // By-value arguments are passed as-is
            DatatypePassedBy::Value(_) => {
                if is_param {
                    self.name.render(src);
                    src.push_str(": ");
                }
                self.type_.render(src);
            }
            // Pointer arguments are passed with a `*mut` out in front
            DatatypePassedBy::Pointer => {
                if is_param {
                    self.name.render(src);
                    src.push_str(": ");
                }
                src.push_str("*mut ");
                self.type_.render(src);
            }
            // ... and pointer/length arguments are passed with first their
            // pointer and then their length, as the name would otherwise imply
            DatatypePassedBy::PointerLengthPair => {
                assert!(is_param);
                src.push_str(self.name.as_str());
                src.push_str("_ptr");
                src.push_str(": ");
                src.push_str("*const ");
                match resolve(&self.type_) {
                    DatatypeIdent::Array(x) => x.render(src),
                    DatatypeIdent::Builtin(BuiltinType::String) => src.push_str("u8"),
                    x => panic!("unexpected pointer length pair type {:?}", x),
                }
                src.push_str(", ");
                src.push_str(self.name.as_str());
                src.push_str("_len");
                src.push_str(": ");
                src.push_str("usize");
            }
        }
    }
}

impl Render for Id {
    fn render(&self, src: &mut String) {
        match self.as_str() {
            "in" => src.push_str("r#in"),
            "type" => src.push_str("r#type"),
            s => src.push_str(s),
        }
    }
}

fn resolve(ty: &DatatypeIdent) -> &DatatypeIdent {
    if let DatatypeIdent::Ident(i) = ty {
        if let DatatypeVariant::Alias(a) = &i.variant {
            return resolve(&a.to);
        }
    }
    return ty;
}
