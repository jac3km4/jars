use cafebabe::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use crate::descriptor::Descriptor;

/// A pattern used to find classes in a JAR file.
///
/// Typically this would represent an obfuscated class.
#[derive(Debug)]
pub struct ClassPat {
    pub(crate) flags: ClassAccessFlags,
    pub(crate) members: Vec<MemberPat>,
    pub(crate) base: Option<TypePat>,
    pub(crate) impls: Vec<TypePat>,
}

impl ClassPat {
    /// Creates a pattern that matches any interface.
    pub fn interface() -> Self {
        let mut this = Self::default();
        this.flags |= ClassAccessFlags::INTERFACE;
        this
    }

    /// Extends the pattern to match any public class.
    #[inline]
    pub fn public(mut self) -> Self {
        self.flags |= ClassAccessFlags::PUBLIC;
        self
    }

    /// Extends the pattern to match any final class.
    #[inline]
    pub fn final_(mut self) -> Self {
        self.flags |= ClassAccessFlags::FINAL;
        self
    }

    /// Extends the pattern to match any abstract class.
    #[inline]
    pub fn abstract_(mut self) -> Self {
        self.flags |= ClassAccessFlags::ABSTRACT;
        self
    }

    /// Extends the pattern with a [`TypePat`],
    /// which will be used to filter on the base class.
    #[inline]
    pub fn with_base(mut self, base: TypePat) -> Self {
        self.base = Some(base);
        self
    }

    /// Extends a pattern with a [`TypePat`],
    /// which will be used to filter on the implemented interfaces.
    #[inline]
    pub fn with_impl(mut self, interface: TypePat) -> Self {
        self.impls.push(interface);
        self
    }

    /// Extends a pattern with a [`MemberPat`],
    /// which will be used to match a class member.
    ///
    /// Calls to this method must follow the same order as
    /// the order in which the members are defined in.
    #[inline]
    pub fn with(mut self, member: MemberPat) -> Self {
        self.members.push(member);
        self
    }
}

impl Default for ClassPat {
    fn default() -> Self {
        Self {
            flags: ClassAccessFlags::empty(),
            members: vec![],
            base: None,
            impls: vec![],
        }
    }
}

/// A pattern used to match on class members.
#[derive(Debug)]
pub enum MemberPat {
    Method {
        flags: MethodAccessFlags,
        param_types: Vec<TypePat>,
        ret_type: TypePat,
    },
    Field {
        flags: FieldAccessFlags,
        field_type: TypePat,
    },
}

/// A pattern used to match on types.
#[derive(Debug)]
pub enum TypePat {
    /// Matches on any type.
    Any,
    /// Matches on void only.
    Void,
    /// Matches on the specified [`Descriptor`].
    Match(Descriptor<'static>),
}

impl TypePat {
    pub fn class_name(&self) -> Option<&'static str> {
        if let Self::Match(Descriptor::Object(obj)) = self {
            Some(obj)
        } else {
            None
        }
    }
}

#[macro_export]
macro_rules! method_mods {
    ($($ident:ident)*) => {
        $crate::paste::paste!($($crate::cafebabe::MethodAccessFlags::[<$ident:upper>])|*)
    };
}

#[macro_export]
macro_rules! field_mods {
    ($($ident:ident)*) => {
        $crate::paste::paste!($($crate::cafebabe::FieldAccessFlags::[<$ident:upper>])|*)
    };
}

/// Macro used as a shorthand to create method patterns.
///
/// The macro expects a sequence of modifiers followed by a Rust function type,
/// which will be translated into a Java method signature using [`HasDescriptor`].
///
/// # Examples
/// ```
/// use jars::method;
///
/// method!(public static (String) -> i32);
/// ```
/// The example above maps to `public static int method(String str)` in Java.
#[macro_export]
macro_rules! method {
    ($($mod:ident)* ($($arg:ty),*) -> $ret:ty) => {
        $crate::MemberPat::Method {
            flags: $crate::method_mods!($($mod)*),
            param_types: vec![$(<$arg as $crate::HasTypePat>::pattern()),*],
            ret_type: <$ret as $crate::HasTypePat>::pattern()
        }
    }
}

/// Macro used as a shorthand to create field patterns.
///
/// # Examples
/// ```
/// use jars::field;
///
/// field!([public] i32);
/// ```
#[macro_export]
macro_rules! field {
    ($typ:ty) => {
        $crate::MemberPat::Field {
            flags: $crate::cafebabe::FieldAccessFlags::empty(),
            field_type: <$typ as $crate::HasTypePat>::pattern()
        }
    };
    ([$($mod:ident)*] $typ:ty) => {
        $crate::MemberPat::Field {
            flags: $crate::field_mods!($($mod)*),
            field_type: <$typ as $crate::HasTypePat>::pattern()
        }
    }
}

/// Type used as a wildcard (matches any type).
pub struct Any;

pub trait HasTypePat {
    fn pattern() -> TypePat;
}

impl<A: HasDescriptor> HasTypePat for A {
    #[inline]
    fn pattern() -> TypePat {
        TypePat::Match(A::descriptor())
    }
}

impl HasTypePat for Any {
    #[inline]
    fn pattern() -> TypePat {
        TypePat::Any
    }
}

impl HasTypePat for () {
    #[inline]
    fn pattern() -> TypePat {
        TypePat::Void
    }
}

pub trait HasDescriptor {
    fn descriptor() -> Descriptor<'static>;
}

impl<A: HasDescriptor> HasDescriptor for &[A] {
    #[inline]
    fn descriptor() -> Descriptor<'static> {
        Descriptor::Array(A::descriptor().into())
    }
}

macro_rules! desc_impl {
    ($ty:ty, $val:expr) => {
        impl HasDescriptor for $ty {
            #[inline]
            fn descriptor() -> Descriptor<'static> {
                $val
            }
        }
    };
}

desc_impl!(bool, Descriptor::Boolean);
desc_impl!(i8, Descriptor::Byte);
desc_impl!(i16, Descriptor::Short);
desc_impl!(i32, Descriptor::Integer);
desc_impl!(i64, Descriptor::Long);
desc_impl!(f32, Descriptor::Float);
desc_impl!(f64, Descriptor::Double);
desc_impl!(char, Descriptor::Char);
desc_impl!(String, Descriptor::Object("java/lang/String"));

pub mod java {
    use super::*;

    // lava lang stuff
    pub struct Boolean;
    desc_impl!(Boolean, Descriptor::Object("java/lang/Boolean"));
    pub struct Byte;
    desc_impl!(Byte, Descriptor::Object("java/lang/Byte"));
    pub struct Short;
    desc_impl!(Short, Descriptor::Object("java/lang/Short"));
    pub struct Integer;
    desc_impl!(Integer, Descriptor::Object("java/lang/Integer"));
    pub struct Long;
    desc_impl!(Long, Descriptor::Object("java/lang/Long"));
    pub struct Float;
    desc_impl!(Float, Descriptor::Object("java/lang/Float"));
    pub struct Double;
    desc_impl!(Double, Descriptor::Object("java/lang/Double"));
    pub struct Character;
    desc_impl!(Character, Descriptor::Object("java/lang/Character"));
    pub struct Iterable;
    desc_impl!(Iterable, Descriptor::Object("java/lang/Iterable"));
    pub struct Runnable;
    desc_impl!(Runnable, Descriptor::Object("java/lang/Runnable"));
    pub struct Object;
    desc_impl!(Object, Descriptor::Object("java/lang/Object"));
    pub struct Throwable;
    desc_impl!(Throwable, Descriptor::Object("java/lang/Throwable"));
    pub struct Thread;
    desc_impl!(Thread, Descriptor::Object("java/lang/Thread"));

    pub struct List;
    desc_impl!(List, Descriptor::Object("java/util/List"));
    pub struct Collection;
    desc_impl!(Collection, Descriptor::Object("java/util/Collection"));
}
