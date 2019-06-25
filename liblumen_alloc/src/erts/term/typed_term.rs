use core::cmp;
use core::mem;

use crate::borrow::CloneToProcess;

use super::*;

/// Concrete `Term` types, i.e. resolved to concrete values, or pointers to values.
///
/// Most runtime code should use this, or the types referenced here,
/// rather than working with raw `Term`s.
///
/// In some cases, these types contain pointers to `Term`, this is primarily the
/// container types, but to properly work on these containers, you must resolve the
/// inner types as well. In these situations, the pointer is _not_ the tagged value,
/// instead, you must dereference the pointer as `Term` and ask it to resolve itself
/// to its typed form.
#[derive(Debug)]
pub enum TypedTerm {
    List(Boxed<Cons>),
    Tuple(Boxed<Tuple>),
    Map(Boxed<MapHeader>),
    Boxed(Term),
    Literal(Term),
    Pid(Pid),
    Port(Port),
    Reference(Reference),
    ExternalPid(Boxed<ExternalPid>),
    ExternalPort(Boxed<ExternalPort>),
    ExternalReference(Boxed<ExternalReference>),
    SmallInteger(SmallInteger),
    BigInteger(Boxed<BigInteger>),
    Float(Float),
    Atom(Atom),
    ProcBin(ProcBin),
    HeapBinary(HeapBin),
    SubBinary(SubBinary),
    MatchContext(MatchContext),
    Closure(Boxed<Closure>),
    Catch,
    Nil,
    None,
}
impl TypedTerm {
    #[inline]
    pub fn is_none(&self) -> bool {
        self.eq(&Self::None)
    }

    #[inline]
    pub fn is_nil(&self) -> bool {
        self.eq(&Self::Nil)
    }

    #[inline]
    pub fn is_catch(&self) -> bool {
        self.eq(&Self::Catch)
    }

    #[inline]
    pub fn is_number(&self) -> bool {
        match self {
            &Self::Float(_) => true,
            &Self::SmallInteger(_) => true,
            &Self::BigInteger(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_binary(&self) -> bool {
        match self {
            &Self::ProcBin(_) => true,
            &Self::HeapBinary(_) => true,
            &Self::SubBinary(_) => true,
            &Self::MatchContext(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_function(&self) -> bool {
        if let &Self::Closure(_) = self {
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn is_pid(&self) -> bool {
        match self {
            &Self::Pid(_) => true,
            &Self::ExternalPid(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_reference(&self) -> bool {
        match self {
            &Self::Reference(_) => true,
            &Self::ExternalReference(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_list(&self) -> bool {
        if let &Self::List(_) = self {
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn is_tuple(&self) -> bool {
        if let &Self::Tuple(_) = self {
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn is_map(&self) -> bool {
        if let &Self::Map(_) = self {
            true
        } else {
            false
        }
    }
}

macro_rules! partial_eq_impl_boxed {
    ($input:expr => $($variant:path),*) => {
        $(
            if let (&$variant(ref lhs), &$variant(ref rhs)) = $input {
                return lhs.eq(rhs);
            }
        )*

        return false;
    }
}

impl PartialEq<TypedTerm> for TypedTerm {
    fn eq(&self, other: &Self) -> bool {
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match (self, other) {
            (&Self::Catch, &Self::Catch) => true,
            (&Self::Nil, &Self::Nil) => true,
            (&Self::None, &Self::None) => true,
            boxed => {
                partial_eq_impl_boxed! { boxed =>
                    Self::List,
                    Self::Tuple,
                    Self::Map,
                    Self::Pid,
                    Self::Port,
                    Self::Reference,
                    Self::ExternalPid,
                    Self::ExternalPort,
                    Self::ExternalReference,
                    Self::BigInteger,
                    Self::ProcBin,
                    Self::HeapBinary,
                    Self::SubBinary,
                    Self::MatchContext,
                    Self::Closure,
                    Self::Boxed,
                    Self::Literal,
                    Self::SmallInteger,
                    Self::Float,
                    Self::Atom
                }
            }
        }
    }
}
impl Eq for TypedTerm {}

macro_rules! partial_cmp_impl {
    (@try_with_equiv $input:expr => $variant:path as [$($equiv:path),*] , $($rest:tt)*) => {
        match $input {
            (&$variant(ref lhs), &$variant(ref rhs)) => { return lhs.partial_cmp(rhs); }
            $(
                (&$variant(ref lhs), &$equiv(ref rhs)) => { return lhs.partial_cmp(rhs); }
                (&$equiv(ref lhs), &$variant(ref rhs)) => { return lhs.partial_cmp(rhs); }
                (&$equiv(ref lhs), &$equiv(ref rhs)) => { return lhs.partial_cmp(rhs); }
            )*
            (&$variant(_), _) => { return Some(Ordering::Greater); }
            $(
                (&$equiv(_), _) => { return Some(Ordering::Greater); }
            )*
            _ => ()
        }

        partial_cmp_impl!(@try_is_constant $input => $($rest)*);
    };
    (@try_with_equiv $input:expr => $($rest:tt)*) => {
        partial_cmp_impl!(@try_without_equiv $input => $($rest)*);
    };
    (@try_with_equiv $input:expr => ) => {
        (());
    };
    (@try_without_equiv $input:expr => $variant:path , $($rest:tt)*) => {
        match $input {
            (&$variant(ref lhs), &$variant(ref rhs)) => { return lhs.partial_cmp(rhs); }
            (&$variant(_), _) => { return Some(Ordering::Greater); }
            _ => ()
        }

        partial_cmp_impl!(@try_is_constant $input => $($rest)*);
    };
    (@try_without_equiv $input:expr => ) => {
        (());
    };
    (@try_is_constant $input:expr => $variant:path where constant , $($rest:tt)*) => {
        match $input {
            (&$variant, &$variant) => return Some(Ordering::Equal),
            (&$variant, _) => return Some(Ordering::Greater),
            _ => ()
        }

        partial_cmp_impl!(@try_is_constant $input => $($rest)*);
    };
    (@try_is_constant $input:expr => $($rest:tt)*) => {
        partial_cmp_impl!(@try_is_invalid $input => $($rest)*);
    };
    (@try_is_invalid $input:expr => $variant:path where invalid , $($rest:tt)*) => {
        if let (&$variant, _) = $input {
            return None;
        }
        if let (_, &$variant) = $input {
            return None;
        }

        partial_cmp_impl!(@try_is_constant $input => $($rest)*);
    };
    (@try_is_invalid $input:expr => $($rest:tt)*) => {
        partial_cmp_impl!(@try_with_equiv $input => $($rest)*);
    };
    (($lhs:expr, $rhs:expr) => $($rest:tt)*) => {
        let input = ($lhs, $rhs);
        partial_cmp_impl!(@try_is_constant input => $($rest)*);

        // Fallback
        // Flip the arguments, then invert the result to avoid duplicating the above
        match $rhs.partial_cmp($lhs) {
            Some(Ordering::Greater) => Some(Ordering::Less),
            Some(Ordering::Less) => Some(Ordering::Greater),
            same => same
        }
    };
}

impl PartialOrd<TypedTerm> for TypedTerm {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        use core::cmp::Ordering;
        // number < atom < reference < fun < port < pid < tuple < map < nil < list < bit string
        partial_cmp_impl! { (self, other) =>
            Self::Catch where invalid,
            Self::None where invalid,
            Self::ProcBin as [Self::HeapBinary, Self::SubBinary, Self::MatchContext],
            Self::List,
            Self::Nil where constant,
            Self::Map,
            Self::Tuple,
            Self::ExternalPid,
            Self::Pid,
            Self::ExternalPort,
            Self::Port,
            Self::Closure,
            Self::ExternalReference,
            Self::Reference,
            Self::Atom,
            Self::BigInteger,
            Self::Float as [Self::SmallInteger],
        }
    }
}

unsafe impl AsTerm for TypedTerm {
    unsafe fn as_term(&self) -> Term {
        match self {
            &Self::List(ref inner) => inner.as_term(),
            &Self::Tuple(ref inner) => inner.as_term(),
            &Self::Map(ref inner) => inner.as_term(),
            &Self::Boxed(ref inner) => Term::make_boxed(inner),
            &Self::Literal(ref inner) => Term::make_boxed_literal(inner),
            &Self::Pid(ref inner) => inner.as_term(),
            &Self::Port(ref inner) => inner.as_term(),
            &Self::Reference(ref inner) => inner.as_term(),
            &Self::ExternalPid(ref inner) => inner.as_term(),
            &Self::ExternalPort(ref inner) => inner.as_term(),
            &Self::ExternalReference(ref inner) => inner.as_term(),
            &Self::SmallInteger(ref inner) => inner.as_term(),
            &Self::BigInteger(ref inner) => inner.as_term(),
            &Self::Float(ref inner) => inner.as_term(),
            &Self::Atom(ref inner) => inner.as_term(),
            &Self::ProcBin(ref inner) => inner.as_term(),
            &Self::HeapBinary(ref inner) => inner.as_term(),
            &Self::SubBinary(ref inner) => inner.as_term(),
            &Self::MatchContext(ref inner) => inner.as_term(),
            &Self::Closure(ref inner) => inner.as_term(),
            &Self::Catch => Term::CATCH,
            &Self::Nil => Term::NIL,
            &Self::None => Term::NONE,
        }
    }
}

impl CloneToProcess for TypedTerm {
    fn clone_to_process<A: AllocInProcess>(&self, process: &mut A) -> Term {
        // Immediates are just copied and returned, all other terms
        // are expected to require allocation, so we delegate to those types
        match self {
            &Self::List(ref inner) => inner.clone_to_process(process),
            &Self::Tuple(ref inner) => inner.clone_to_process(process),
            &Self::Map(ref inner) => inner.clone_to_process(process),
            &Self::Boxed(ref inner) => inner.clone_to_process(process),
            &Self::Literal(inner) => inner,
            &Self::Pid(inner) => unsafe { inner.as_term() },
            &Self::Port(inner) => unsafe { inner.as_term() },
            &Self::Reference(inner) => unsafe { inner.as_term() },
            &Self::ExternalPid(ref inner) => inner.clone_to_process(process),
            &Self::ExternalPort(ref inner) => inner.clone_to_process(process),
            &Self::ExternalReference(ref inner) => inner.clone_to_process(process),
            &Self::SmallInteger(inner) => unsafe { inner.as_term() },
            &Self::BigInteger(ref inner) => inner.clone_to_process(process),
            &Self::Float(inner) => unsafe { inner.as_term() },
            &Self::Atom(inner) => unsafe { inner.as_term() },
            &Self::ProcBin(ref inner) => inner.clone_to_process(process),
            &Self::HeapBinary(ref inner) => inner.clone_to_process(process),
            &Self::SubBinary(ref inner) => inner.clone_to_process(process),
            &Self::MatchContext(ref inner) => inner.clone_to_process(process),
            &Self::Closure(ref inner) => inner.clone_to_process(process),
            &Self::Catch => Term::CATCH,
            &Self::Nil => Term::NIL,
            &Self::None => Term::NONE,
        }
    }
}