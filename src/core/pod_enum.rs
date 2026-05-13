macro_rules! pod_enum {
    (pub enum $struct_name: ident { $($variant:ident),* $(,)? }) => {
        pod_enum! {
            #[derive()]
            pub enum $struct_name {
                $($variant),*
            }
        }
    };
    (#[derive($($derive:ident),*)]pub enum $struct_name: ident { $($variant:ident),* $(,)? }) => {
        #[derive($($derive,)* Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(u8)]
        pub enum $struct_name {
            $($variant),*
        }
        impl $struct_name {
            pub const MIN: Self = Self::from_int(0).unwrap();
            pub const MAX: Self = Self::from_int(Self::LEN as u8 - 1).unwrap();
            pub const LEN: usize = ${count($variant)};

            #[must_use]
            pub const fn from_int(int: u8) -> Option<Self> {
                match int {
                    ..${count($variant)} => Some(unsafe { Self::from_int_unchecked(int) }),
                    _ => None,
                }
            }
            #[track_caller]
            #[must_use]
            /// # Safety
            /// int must be <= `Self::MAX`
            pub const unsafe fn from_int_unchecked(int: u8) -> Self {
                debug_assert!((int as usize) < Self::LEN);
                unsafe { std::mem::transmute(int) }
            }
            #[must_use]
            pub const fn add_int(self, rhs: u8) -> Option<Self> {
                Self::from_int(self as u8 + rhs)
            }
            #[must_use]
            pub const fn sub_int(self, rhs: u8) -> Option<Self> {
                Self::from_int((self as u8).wrapping_sub(rhs))
            }
            #[must_use]
            pub const fn add_int_signed(self, rhs: i8) -> Option<Self> {
                let out = self as i8 + rhs;
                if (out >= 0 && out < Self::LEN as i8) {
                    Some(unsafe { Self::from_int_unchecked(out as u8) })
                } else {
                    None
                }
            }
            #[must_use]
            #[expect(clippy::missing_safety_doc, reason="TODO")]
            pub const unsafe fn add_int_signed_unchecked(self, rhs: i8) -> Self {
                unsafe { Self::from_int_unchecked((self as u8).wrapping_add_signed(rhs)) }
            }
            #[track_caller]
            #[must_use]
            #[expect(clippy::missing_safety_doc, reason="TODO")]
            pub const unsafe fn add_int_unchecked(self, rhs: u8) -> Self {
                debug_assert!(self as u8 + rhs < Self::LEN as u8);
                unsafe { Self::from_int_unchecked(self as u8 + rhs) }
            }
            #[track_caller]
            #[must_use]
            #[expect(clippy::missing_safety_doc, reason="TODO")]
            pub const unsafe fn sub_int_unchecked(self, rhs: u8) -> Self {
                unsafe { Self::from_int_unchecked(self as u8 - rhs) }
            }
            #[must_use]
            /// Inverts self; computing `Self::MAX` - self
            pub const fn invert(self) -> Self {
                unsafe { Self::from_int_unchecked(Self::MAX as u8 - self as u8) }
            }
        }

    impl<T> ::core::ops::Index<$struct_name> for [T; $struct_name::LEN] {
        type Output = T;
        fn index(&self, index: $struct_name) -> &T {
            unsafe { self.get_unchecked(index as usize) }
        }
    }

    impl<T> ::core::ops::IndexMut<$struct_name> for [T; $struct_name::LEN] {
        fn index_mut(&mut self, index: $struct_name) -> &mut T {
            unsafe { self.get_unchecked_mut(index as usize) }
        }
    }
}}
