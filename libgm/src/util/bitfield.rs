macro_rules! bitfield_struct {
    (
        $(#[$meta:meta])*
        $name:ident : $int:ty {
            $(
                $(#[$field_meta:meta])*
                $field:ident: $bit:literal
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Default, PartialEq, Eq)]
        pub struct $name {
            $(
                $(#[$field_meta])*
                pub $field: bool,
            )*
        }

        impl crate::gamemaker::elements::GMElement for $name {
            fn deserialize(reader: &mut crate::gamemaker::deserialize::reader::DataReader) -> Result<Self> {
                let raw = <$int>::deserialize(reader)?;
                Ok(Self::parse(raw))
            }

            fn serialize(&self, builder: &mut crate::gamemaker::serialize::builder::DataBuilder) -> Result<()> {
                self.build().serialize(builder)
            }
        }

        impl $name {
            pub(crate) const fn parse(raw: $int) -> Self {
                Self {
                    $($field: raw & (1 << $bit) != 0,)*
                }
            }

            pub(crate) const fn build(&self) -> $int {
                let mut raw = 0;
                $(
                    if self.$field {
                        raw |= 1 << $bit;
                    }
                )*
                raw
            }
        }
    };
}

pub(crate) use bitfield_struct;
