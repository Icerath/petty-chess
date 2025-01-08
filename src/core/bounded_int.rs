macro_rules! impl_int_getters {
    ($ty: ident) => {
        #[must_use]
        pub const fn $ty(self) -> $ty {
            self.u8() as $ty
        }
    };
    ($($ty: ident),+) => {
        $(impl_int_getters!($ty);)+
    };
}

macro_rules! impl_into {
    ($struct_name: ident, $ty: ident) => {
        impl From<$struct_name> for $ty {
            #[must_use]
            fn from(value: $struct_name) -> Self {
                value.$ty()
            }
        }
    };
    ($struct_name: ident : $($ty: ident),+) => {
        $(impl_into!($struct_name, $ty);)+
    };
}

macro_rules! impl_try_from {
    ($struct_name: ident, $max: literal, $ty: ident) => {
        impl TryFrom<$ty> for $struct_name {
            type Error = ();
            fn try_from(value: $ty) -> Result<Self, Self::Error> {
                match value {
                    0..$max => Ok(Self(value as u8)),
                    _ => Err(())
                }
            }
        }
    };
    ($struct_name: ident $max: literal : $($ty: ident),+) => {
        $(impl_try_from!($struct_name, $max, $ty);)+
    };
}

macro_rules! bounded_int {
    (pub struct $struct_name: ident { $max: literal }) => {
        #[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $struct_name(u8);
        impl $struct_name {
            pub const MIN: Self = Self(0);
            pub const MAX: Self = Self($max - 1);

            #[must_use]
                        pub const fn from_int(int: u8) -> Option<Self> {
                match int {
                    ..$max => Some(Self(int)),
                    _ => None,
                }
            }
            #[track_caller]
            #[must_use]
            /// # Safety
            /// int must be less than `Self::MAX`
            pub const unsafe fn from_int_unchecked(int: u8) -> Self {
                debug_assert!(int < $max);
                Self(int)
            }
            #[must_use]
            pub const fn add_int(self, rhs: u8) -> Option<Self> {
                match self.0 + rhs {
                    n @ ..$max => Some(Self(n)),
                    _ => None,

                }
            }
            #[must_use]
            pub const fn sub_int(self, rhs: u8) -> Option<Self> {
                match self.u8().wrapping_sub(rhs) {
                    n @ ..$max => Some(Self(n)),
                    _ => None,

                }
            }
            #[must_use]
            pub const fn add_int_signed(self, rhs: i8) -> Option<Self> {
                let out = self.0 as i8 + rhs;
                if (out >= 0 && out < $max) {
                    Some(Self(out as u8))
                } else {
                    None
                }
            }
            #[must_use]
            #[expect(clippy::missing_safety_doc, reason="TODO")]
            pub const unsafe fn add_int_signed_unchecked(self, rhs: i8) -> Self {
                Self(self.0.wrapping_add_signed(rhs))
            }
            #[track_caller]
            #[must_use]
            #[expect(clippy::missing_safety_doc, reason="TODO")]
            pub const unsafe fn add_int_unchecked(self, rhs: u8) -> Self {
                debug_assert!(self.0 + rhs < $max);
                Self(self.0 + rhs)
            }
            #[track_caller]
            #[must_use]
            #[expect(clippy::missing_safety_doc, reason="TODO")]
            pub const unsafe fn sub_int_unchecked(self, rhs: u8) -> Self {
                Self(self.0 - rhs)
            }
            #[must_use]
            pub const fn u8(self) -> u8 {
                unsafe { ::std::hint::assert_unchecked(self.0 < $max) };
                self.0
            }
            impl_int_getters!(i8, i16, i32, i64, isize, u16, u32, u64, usize);
        }
        impl_into!($struct_name : u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);
        impl_try_from!($struct_name $max : u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);
    };
}
