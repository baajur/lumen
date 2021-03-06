use std::convert::TryInto;
use std::panic;

use liblumen_alloc::erts::term::prelude::*;
use liblumen_core::sys::Endianness;

use crate::process::current_process;

#[export_name = "__lumen_builtin_self"]
pub extern "C" fn builtin_self() -> Term {
    current_process().pid_term()
}

/// Strict equality
#[export_name = "__lumen_builtin_cmp.eq"]
pub extern "C" fn builtin_cmpeq(lhs: Term, rhs: Term) -> bool {
    let result = panic::catch_unwind(|| {
        if let Ok(left) = lhs.decode() {
            if let Ok(right) = rhs.decode() {
                left.exact_eq(&right)
            } else {
                false
            }
        } else {
            if lhs.is_none() && rhs.is_none() {
                true
            } else {
                false
            }
        }
    });
    if let Ok(res) = result {
        res
    } else {
        false
    }
}

#[export_name = "__lumen_builtin_cmp.neq"]
pub extern "C" fn builtin_cmpneq(lhs: Term, rhs: Term) -> bool {
    !builtin_cmpeq(lhs, rhs)
}

macro_rules! comparison_builtin {
    ($name:expr, $alias:ident, $op:tt) => {
        #[export_name = $name]
        pub extern "C" fn $alias(lhs: Term, rhs: Term) -> bool {
            let result = panic::catch_unwind(|| {
                if let Ok(left) = lhs.decode() {
                    if let Ok(right) = rhs.decode() {
                        return left $op right;
                    }
                }
                false
            });
            if let Ok(res) = result {
                res
            } else {
                false
            }
        }
    }
}

comparison_builtin!("__lumen_builtin_cmp.lt",  builtin_cmp_lt,  <);
comparison_builtin!("__lumen_builtin_cmp.lte", builtin_cmp_lte, <=);
comparison_builtin!("__lumen_builtin_cmp.gt",  builtin_cmp_gt,  >);
comparison_builtin!("__lumen_builtin_cmp.gte", builtin_cmp_gte, >=);

macro_rules! math_builtin {
    ($name:expr, $alias:ident, $trait:tt, $op:ident) => {
        #[export_name = $name]
        pub extern "C" fn $alias(lhs: Term, rhs: Term) -> Term {
            use std::ops::*;
            let result = panic::catch_unwind(|| {
                let l = lhs.decode().unwrap();
                let r = rhs.decode().unwrap();
                match (l, r) {
                    (TypedTerm::SmallInteger(li), TypedTerm::SmallInteger(ri)) => {
                        current_process().integer(li.$op(ri)).unwrap()
                    }
                    (TypedTerm::SmallInteger(li), TypedTerm::Float(ri)) => {
                        let li: f64 = li.into();
                        let f = <f64 as $trait<f64>>::$op(li, ri.value());
                        current_process().float(f).unwrap()
                    }
                    (TypedTerm::SmallInteger(li), TypedTerm::BigInteger(ri)) => {
                        let li: BigInteger = li.into();
                        current_process().integer(li.$op(ri.as_ref())).unwrap()
                    }
                    (TypedTerm::Float(li), TypedTerm::Float(ri)) => {
                        let f = <f64 as $trait<f64>>::$op(li.value(), ri.value());
                        current_process().float(f).unwrap()
                    }
                    (TypedTerm::Float(li), TypedTerm::SmallInteger(ri)) => {
                        let ri: f64 = ri.into();
                        let f = <f64 as $trait<f64>>::$op(li.value(), ri);
                        current_process().float(f).unwrap()
                    }
                    (TypedTerm::Float(li), TypedTerm::BigInteger(ri)) => {
                        let ri: f64 = ri.as_ref().into();
                        let f = <f64 as $trait<f64>>::$op(li.value(), ri);
                        current_process().float(f).unwrap()
                    }
                    (TypedTerm::BigInteger(li), TypedTerm::SmallInteger(ri)) => {
                        let ri: BigInteger = ri.into();
                        current_process().integer(li.as_ref().$op(ri)).unwrap()
                    }
                    (TypedTerm::BigInteger(li), TypedTerm::Float(ri)) => {
                        let li: f64 = li.as_ref().into();
                        let f = <f64 as $trait<f64>>::$op(li, ri.value());
                        current_process().float(f).unwrap()
                    }
                    (TypedTerm::BigInteger(li), TypedTerm::BigInteger(ri)) => {
                        current_process().integer(li.$op(ri)).unwrap()
                    }
                    _ => panic!("expected numeric argument to builtin '{}'", $name),
                }
            });
            if let Ok(res) = result {
                res
            } else {
                Term::NONE
            }
        }
    }
}

macro_rules! integer_math_builtin {
    ($name:expr, $alias:ident, $op:ident) => {
        #[export_name = $name]
        pub extern "C" fn $alias(lhs: Term, rhs: Term) -> Term {
            use std::ops::*;
            let result = panic::catch_unwind(|| {
                let l = lhs.decode().unwrap();
                let r = rhs.decode().unwrap();
                let li: Integer = l.try_into().unwrap();
                let ri: Integer = r.try_into().unwrap();
                let result = li.$op(ri);
                current_process().integer(result).unwrap()
            });
            if let Ok(res) = result {
                res
            } else {
                Term::NONE
            }
        }
    }
}

math_builtin!("__lumen_builtin_math.add", builtin_math_add, Add, add);
math_builtin!("__lumen_builtin_math.sub", builtin_math_sub, Sub, sub);
math_builtin!("__lumen_builtin_math.mul", builtin_math_mul, Mul, mul);
math_builtin!("__lumen_builtin_math.fdiv", builtin_math_fdiv, Div, div);

integer_math_builtin!("__lumen_builtin_math.div", builtin_math_div, div);
integer_math_builtin!("__lumen_builtin_math.rem", builtin_math_rem, rem);
integer_math_builtin!("__lumen_builtin_math.bsl", builtin_math_bsl, shl);
integer_math_builtin!("__lumen_builtin_math.bsr", builtin_math_bsr, shr);
integer_math_builtin!("__lumen_builtin_math.band", builtin_math_band, bitand);
integer_math_builtin!("__lumen_builtin_math.bor", builtin_math_bor, bitor);
integer_math_builtin!("__lumen_builtin_math.bxor", builtin_math_bxor, bitxor);

/// Capture the current stack trace
#[export_name = "__lumen_builtin_trace_capture"]
pub extern "C" fn builtin_trace_capture() -> Term {
    // TODO:
    Term::NIL
}

/// Binary Construction
#[export_name = "__lumen_builtin_binary_start"]
pub extern "C" fn builtin_binary_start() -> *mut BinaryBuilder {
    let builder = Box::new(BinaryBuilder::new());
    Box::into_raw(builder)
}

#[export_name = "__lumen_builtin_binary_finish"]
pub extern "C" fn builtin_binary_finish(builder: *mut BinaryBuilder) -> Term {
    let builder = unsafe { Box::from_raw(builder) };
    let bytes = builder.finish();
    // TODO: Need to properly handle cases where heap runs out of space
    current_process()
        .binary_from_bytes(bytes.as_slice())
        .unwrap()
}

#[export_name = "__lumen_builtin_binary_push_integer"]
pub extern "C" fn builtin_binary_push_integer(
    builder: &mut BinaryBuilder,
    value: Term,
    size: Term,
    unit: u8,
    signed: bool,
    endianness: Endianness,
) -> BinaryPushResult {
    let tt = value.decode().unwrap();
    let val: Result<Integer, _> = tt.try_into();
    let result = if let Ok(i) = val {
        let flags = BinaryPushFlags::new(signed, endianness);
        let bit_size = calculate_bit_size(size, unit, flags).unwrap();
        builder.push_integer(i, bit_size, flags)
    } else {
        Err(())
    };
    BinaryPushResult {
        builder,
        success: result.is_ok(),
    }
}

#[export_name = "__lumen_builtin_binary_push_float"]
pub extern "C" fn builtin_binary_push_float(
    builder: &mut BinaryBuilder,
    value: Term,
    size: Term,
    unit: u8,
    signed: bool,
    endianness: Endianness,
) -> BinaryPushResult {
    let tt = value.decode().unwrap();
    let val: Result<Float, _> = tt.try_into();
    let result = if let Ok(f) = val {
        let flags = BinaryPushFlags::new(signed, endianness);
        let bit_size = calculate_bit_size(size, unit, flags).unwrap();
        builder.push_float(f.into(), bit_size, flags)
    } else {
        Err(())
    };
    BinaryPushResult {
        builder,
        success: result.is_ok(),
    }
}

#[export_name = "__lumen_builtin_binary_push_utf8"]
pub extern "C" fn builtin_binary_push_utf8(
    builder: &mut BinaryBuilder,
    value: Term,
) -> BinaryPushResult {
    let tt = value.decode().unwrap();
    let val: Result<SmallInteger, _> = tt.try_into();
    let result = if let Ok(small) = val {
        builder.push_utf8(small.into())
    } else {
        Err(())
    };
    BinaryPushResult {
        builder,
        success: result.is_ok(),
    }
}

#[export_name = "__lumen_builtin_binary_push_utf16"]
pub extern "C" fn builtin_binary_push_utf16(
    builder: &mut BinaryBuilder,
    value: Term,
    signed: bool,
    endianness: Endianness,
) -> BinaryPushResult {
    let tt = value.decode().unwrap();
    let val: Result<SmallInteger, _> = tt.try_into();
    let result = if let Ok(small) = val {
        let flags = BinaryPushFlags::new(signed, endianness);
        builder.push_utf16(small.into(), flags)
    } else {
        Err(())
    };
    BinaryPushResult {
        builder,
        success: result.is_ok(),
    }
}

#[export_name = "__lumen_builtin_binary_push_utf32"]
pub extern "C" fn builtin_binary_push_utf32(
    builder: &mut BinaryBuilder,
    value: Term,
    size: Term,
    unit: u8,
    signed: bool,
    endianness: Endianness,
) -> BinaryPushResult {
    let tt = value.decode().unwrap();
    let result: Result<SmallInteger, _> = tt.try_into();
    if let Ok(small) = result {
        let i: isize = small.into();
        if i > 0x10FFFF || (0xD800 <= i && i <= 0xDFFF) {
            // Invalid utf32 integer
            return BinaryPushResult {
                builder,
                success: false,
            };
        }
        let flags = BinaryPushFlags::new(signed, endianness);
        let bit_size = calculate_bit_size(size, unit, flags).unwrap();
        let success = builder.push_integer(small.into(), bit_size, flags).is_ok();
        BinaryPushResult { builder, success }
    } else {
        BinaryPushResult {
            builder,
            success: false,
        }
    }
}

#[export_name = "__lumen_builtin_binary_push_binary"]
pub extern "C" fn builtin_binary_push_binary(
    builder: &mut BinaryBuilder,
    value: Term,
    size: Term,
    unit: u8,
) -> BinaryPushResult {
    let flags = BinaryPushFlags::default();
    let bit_size = calculate_bit_size(size, unit, flags).unwrap();
    let result = match value.decode().unwrap() {
        TypedTerm::HeapBinary(bin) => builder.push_binary(bin, None, bit_size),
        TypedTerm::ProcBin(bin) => builder.push_binary(bin, None, bit_size),
        TypedTerm::BinaryLiteral(bin) => builder.push_binary(bin, None, bit_size),
        TypedTerm::SubBinary(bin) => {
            builder.push_binary(bin, Some(bin.bit_offset() as usize), bit_size)
        }
        TypedTerm::MatchContext(bin) => builder.push_binary(bin, None, bit_size),
        _ => Err(()),
    };
    BinaryPushResult {
        builder,
        success: result.is_ok(),
    }
}

#[export_name = "__lumen_builtin_binary_push_binary_all"]
pub extern "C" fn builtin_binary_push_binary_all(
    builder: &mut BinaryBuilder,
    value: Term,
    unit: u8,
) -> BinaryPushResult {
    let result = match value.decode().unwrap() {
        TypedTerm::HeapBinary(bin) => builder.push_binary_all(bin, None, unit),
        TypedTerm::ProcBin(bin) => builder.push_binary_all(bin, None, unit),
        TypedTerm::BinaryLiteral(bin) => builder.push_binary_all(bin, None, unit),
        TypedTerm::SubBinary(bin) => {
            builder.push_binary_all(bin, Some(bin.bit_offset() as usize), unit)
        }
        TypedTerm::MatchContext(bin) => builder.push_binary_all(bin, None, unit),
        _ => Err(()),
    };
    BinaryPushResult {
        builder,
        success: result.is_ok(),
    }
}

#[export_name = "__lumen_builtin_binary_push_string"]
pub extern "C" fn builtin_binary_push_string(
    builder: &mut BinaryBuilder,
    buffer: *const u8,
    len: usize,
) -> BinaryPushResult {
    let bytes = unsafe { core::slice::from_raw_parts(buffer, len) };
    let success = builder.push_string(bytes).is_ok();
    BinaryPushResult { builder, success }
}
