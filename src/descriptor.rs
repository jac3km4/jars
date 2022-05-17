use thiserror::Error;

/// A [Java type descriptor](https://docs.oracle.com/javase/specs/jvms/se18/html/jvms-4.html#jvms-4.3.2).
#[derive(Debug, Clone, PartialEq)]
pub enum Descriptor<'a> {
    Boolean,
    Byte,
    Short,
    Integer,
    Long,
    Float,
    Double,
    Char,
    Array(Box<Self>),
    Object(&'a str),
}

impl<'a> Descriptor<'a> {
    /// Attempts to parse a type descriptor, possibly borrowing from the input.
    #[inline]
    pub fn parse(mut str: &'a str) -> Result<Self, DescriptorError> {
        Self::consume(&mut str)
    }

    fn consume(str: &mut &'a str) -> Result<Self, DescriptorError> {
        let char = str.as_bytes().first().ok_or(DescriptorError::EndOfInput)?;
        if !str.is_char_boundary(1) {
            return Err(DescriptorError::InvalidPrefix);
        }
        *str = &str[1..];
        match char {
            b'[' => Ok(Self::Array(Self::consume(str)?.into())),
            b'Z' => Ok(Self::Boolean),
            b'B' => Ok(Self::Byte),
            b'S' => Ok(Self::Short),
            b'I' => Ok(Self::Integer),
            b'J' => Ok(Self::Long),
            b'F' => Ok(Self::Float),
            b'D' => Ok(Self::Double),
            b'C' => Ok(Self::Char),
            b'L' => {
                let (name, rem) = str.split_once(';').ok_or(DescriptorError::MismatchedChar(';'))?;
                *str = rem;
                Ok(Self::Object(name))
            }
            _ => Err(DescriptorError::InvalidPrefix),
        }
    }
}

/// A [Java type signature](https://docs.oracle.com/javase/specs/jvms/se18/html/jvms-4.html#jvms-4.7.9.1).
#[derive(Debug, Clone, PartialEq)]
pub enum Signature<'a> {
    Descriptor(Descriptor<'a>),
    Parametrized(&'a str, Box<[Signature<'a>]>),
}

impl<'a> Signature<'a> {
    /// Attempts to parse a signature, possibly borrowing from the input.
    #[inline]
    pub fn parse(mut str: &'a str) -> Result<Self, DescriptorError> {
        Self::consume(&mut str)
    }

    fn consume(str: &mut &'a str) -> Result<Signature<'a>, DescriptorError> {
        match str.strip_prefix('L').and_then(|str| str.split_once('<')) {
            Some((name, mut rem)) => {
                let mut arguments = vec![];
                while rem.as_bytes().first() != Some(&b'>') {
                    arguments.push(Self::consume(&mut rem)?);
                }
                *str = rem
                    .strip_suffix(">;")
                    .ok_or(DescriptorError::MismatchedChar('>'))?;
                Ok(Self::Parametrized(name, arguments.into_boxed_slice()))
            }
            None => Ok(Self::Descriptor(Descriptor::consume(str)?)),
        }
    }
}

/// A [Java method descriptor](https://docs.oracle.com/javase/specs/jvms/se18/html/jvms-4.html#jvms-4.3.3).
#[derive(Debug)]
pub struct MethodDescriptor<'a> {
    pub return_type: Option<Descriptor<'a>>,
    pub param_types: Vec<Descriptor<'a>>,
}

impl<'a> MethodDescriptor<'a> {
    #[inline]
    fn new(return_type: Option<Descriptor<'a>>, param_types: Vec<Descriptor<'a>>) -> Self {
        Self {
            return_type,
            param_types,
        }
    }

    /// Attempts to parse a method descriptor, possibly borrowing from the input.
    pub fn parse(str: &'a str) -> Result<Self, DescriptorError> {
        let mut rem = str
            .strip_prefix('(')
            .ok_or(DescriptorError::MismatchedChar('('))?;
        let mut params = vec![];
        while rem.as_bytes().first() != Some(&b')') {
            params.push(Descriptor::consume(&mut rem)?);
        }
        rem = &rem[1..];
        let return_type = if rem.as_bytes().first() == Some(&b'V') {
            None
        } else {
            Some(Descriptor::consume(&mut rem)?)
        };
        Ok(Self::new(return_type, params))
    }
}

#[derive(Debug, Error)]
pub enum DescriptorError {
    #[error("unexpected end of input")]
    EndOfInput,
    #[error("expected char {0}")]
    MismatchedChar(char),
    #[error("invalid descriptor prefix character")]
    InvalidPrefix,
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn parse_descriptors() {
        let typ = Descriptor::parse("[B").unwrap();
        assert_eq!(typ, Descriptor::Array(Descriptor::Byte.into()));

        let typ = Descriptor::parse("Ljava/lang/String;").unwrap();
        assert_eq!(typ, Descriptor::Object("java/lang/String"));

        let desc = MethodDescriptor::parse("([BLjava/lang/String;)V").unwrap();
        assert_eq!(desc.return_type, None);
        assert_eq!(desc.param_types, vec![
            Descriptor::Array(Descriptor::Byte.into()),
            Descriptor::Object("java/lang/String"),
        ])
    }

    #[test]
    fn parse_signatures() {
        let desc = Signature::parse("Ljava/util/Map<Ljava/lang/Integer;Ljava/lang/Boolean;>;").unwrap();
        assert_eq!(
            desc,
            Signature::Parametrized(
                "java/util/Map",
                [
                    Signature::Descriptor(Descriptor::Object("java/lang/Integer")),
                    Signature::Descriptor(Descriptor::Object("java/lang/Boolean"))
                ]
                .into()
            )
        );

        let desc = Signature::parse("Ljava/util/LinkedList<LaBi<Ljava/lang/Long;[B>;>;").unwrap();
        assert_eq!(
            desc,
            Signature::Parametrized(
                "java/util/LinkedList",
                [Signature::Parametrized(
                    "aBi",
                    [
                        Signature::Descriptor(Descriptor::Object("java/lang/Long")),
                        Signature::Descriptor(Descriptor::Array(Descriptor::Byte.into()))
                    ]
                    .into()
                )]
                .into()
            )
        )
    }
}
