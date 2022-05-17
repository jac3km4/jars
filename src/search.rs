use std::io;

use cafebabe::ClassFile;
use from_iter::FromIterator;

use crate::descriptor::{Descriptor, MethodDescriptor};
use crate::jar::{Jar, JarEntry};
use crate::pat::{ClassPat, MemberPat, TypePat};
use crate::result::{Error, Result};

/// Searches for the provided patterns in an archive.
///
/// This function allows for more than one match per pattern.
pub fn search_many<R: io::Read + io::Seek>(jar: &mut Jar<R>, pats: &[ClassPat]) -> Result<Vec<Match>> {
    let mut results = vec![];
    for entry in jar.classes() {
        let entry = entry?;
        let class = entry.parse_without_bytecode()?;
        for (i, pat) in pats.iter().enumerate() {
            if check_class(&class, pat).is_some() {
                results.push(Match { entry, pattern: i });
                break;
            }
        }
    }
    Ok(results)
}

/// Searches for the provided patterns in an archive.
///
/// This function expects to find exactly one match per pattern and fails othrwise.
pub fn search_exact<R: io::Read + io::Seek, const N: usize>(
    jar: &mut Jar<R>,
    pats: &[ClassPat; N],
) -> Result<[JarEntry; N]> {
    let mut matches = search_many(jar, pats)?;
    matches.sort_by_key(|mat| mat.pattern);

    if let Some((pat, mat)) = matches.iter().enumerate().find(|(i, m)| *i != m.pattern) {
        if pat > mat.pattern {
            return Err(Error::TooManyMatches(mat.pattern));
        } else {
            return Err(Error::PatternNotFound(pat));
        }
    }

    let res = <[JarEntry; N]>::from_iter(matches.into_iter().map(|mat| mat.entry));
    Ok(res)
}

fn check_class(class: &ClassFile, pat: &ClassPat) -> Option<()> {
    if !class.access_flags.contains(pat.flags) {
        return None;
    }
    match (&pat.base, class.super_class.as_deref()) {
        (None, None) => {}
        (None, Some("java/lang/Object")) => {}
        (Some(TypePat::Any), Some(_)) => {}
        (Some(pat), Some(base)) if pat.class_name()? == base => {}
        _ => return None,
    }

    for (i, pat) in pat.impls.iter().enumerate() {
        if class.interfaces.get(i)? != pat.class_name()? {
            return None;
        }
    }

    let mut methods = class.methods.iter();
    let mut fields = class.fields.iter();

    for member in &pat.members {
        match member {
            MemberPat::Method {
                flags,
                param_types,
                ret_type,
            } => {
                let method = methods.next()?;
                if !method.access_flags.contains(*flags) {
                    return None;
                }

                let descriptor = MethodDescriptor::parse(&method.descriptor).ok()?;
                if descriptor.param_types.len() != param_types.len() {
                    return None;
                }

                match (ret_type, descriptor.return_type) {
                    (TypePat::Void, None) => {}
                    (tp, Some(ty)) => check_type(ty, tp)?,
                    _ => return None,
                }
                for (pat, desc) in param_types.iter().zip(descriptor.param_types) {
                    check_type(desc, pat)?;
                }
            }
            MemberPat::Field { flags, field_type } => {
                let field = fields.next()?;
                if !field.access_flags.contains(*flags) {
                    return None;
                }
                let descriptor = Descriptor::parse(&field.descriptor).ok()?;
                check_type(descriptor, field_type)?;
            }
        }
    }

    if methods.len() > 0 || fields.len() > 0 {
        return None;
    }

    Some(())
}

fn check_type(descriptor: Descriptor, pat: &TypePat) -> Option<()> {
    match pat {
        TypePat::Any => Some(()),
        TypePat::Match(expected) if descriptor == *expected => Some(()),
        _ => None,
    }
}

#[derive(Debug)]
pub struct Match {
    pub entry: JarEntry,
    pub pattern: usize,
}
