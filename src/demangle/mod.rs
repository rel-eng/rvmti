// Copyright 2018 rel-eng
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:
//   1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.
//   2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.
//   3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote products derived from this software without specific prior written permission.
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::fmt;

#[derive(Debug, PartialEq)]
pub struct BinaryName {
    packages: Vec<String>,
    class: String,
}

#[derive(Debug, PartialEq)]
pub struct ClassType {
    name: BinaryName,
}

#[derive(Debug, PartialEq)]
pub enum ScalarFieldType {
    Byte,
    Char,
    Double,
    Float,
    Integer,
    Long,
    Short,
    Boolean,
    Class{name: BinaryName},
}

#[derive(Debug, PartialEq)]
pub struct FieldType {
    scalar_type: ScalarFieldType,
    dimensions: usize,
}

#[derive(Debug, PartialEq)]
pub struct MethodType {
    parameter_types: Vec<FieldType>,
    return_type: Option<FieldType>,
}

impl BinaryName {

    pub fn new(mangled_name: &str) -> Result<BinaryName, DemangleError> {
        if mangled_name.starts_with("/") || mangled_name.ends_with("/") || mangled_name.contains("//") {
            return Err(DemangleError::DemangleFailed);
        }
        if !mangled_name.contains("/") {
            return Ok(BinaryName{packages: Vec::new(), class: mangled_name.to_string()})
        }
        let splitted: Vec<&str> = mangled_name.split("/").collect();
        let (last, head) = splitted.split_last().unwrap();
        return Ok(BinaryName{packages: head.iter().map(|s| s.to_string()).collect(), class: last.to_string()});
    }

    pub fn package_as_file_path(&self, source_file_name: &str) -> String {
        if self.packages.is_empty() {
            source_file_name.to_string()
        } else {
            format!("{}/{}", self.packages.join("/"), source_file_name).to_string()
        }
    }

}

impl fmt::Display for BinaryName {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.packages.is_empty() {
            write!(f, "{}", self.class)
        } else {
            write!(f, "{}.{}", self.packages.join("."), self.class)
        }
    }

}

impl ClassType {

    pub fn new(mangled_class_type: &str) -> Result<ClassType, DemangleError> {
        let len_chars = mangled_class_type.chars().count();
        if !(mangled_class_type.starts_with("L") && mangled_class_type.ends_with(";") && len_chars >= 3) {
            return Err(DemangleError::DemangleFailed);
        }
        let name = BinaryName::new(&mangled_class_type.chars().skip(1).take(len_chars - 2).collect::<String>())?;
        return Ok(ClassType{ name });
    }

    pub fn package_as_file_path(&self, source_file_name: &str) -> String {
        self.name.package_as_file_path(source_file_name)
    }

}

impl fmt::Display for ClassType {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }

}

impl FieldType {

    pub fn new(mangled_field_type: &str) -> Result<FieldType, DemangleError> {
        let mut dimensions = 0 as usize;
        let mut state = FieldTypeParserState::Dimensions;
        let mut char_iter = mangled_field_type.chars();
        let mut class_name = String::from("");
        loop {
            let next_char = char_iter.next();
            match next_char {
                Some(c) => {
                    match state {
                        FieldTypeParserState::Dimensions => {
                            match c {
                                '[' => dimensions += 1 as usize,
                                'B' => state = FieldTypeParserState::TypeTagByte,
                                'C' => state = FieldTypeParserState::TypeTagChar,
                                'D' => state = FieldTypeParserState::TypeTagDouble,
                                'F' => state = FieldTypeParserState::TypeTagFloat,
                                'I' => state = FieldTypeParserState::TypeTagInteger,
                                'J' => state = FieldTypeParserState::TypeTagLong,
                                'S' => state = FieldTypeParserState::TypeTagShort,
                                'Z' => state = FieldTypeParserState::TypeTagBoolean,
                                'L' => state = FieldTypeParserState::TypeTagClass,
                                _ => return Err(DemangleError::DemangleFailed),
                            }
                        },
                        FieldTypeParserState::TypeTagClass => {
                            match c {
                                ';' => state = FieldTypeParserState::ClassNameEnd,
                                _ => class_name.push(c),
                            }

                        },
                        _ => return Err(DemangleError::DemangleFailed),
                    }
                },
                None => break,
            };
        }
        match state {
            FieldTypeParserState::TypeTagByte => return Ok(FieldType{scalar_type: ScalarFieldType::Byte, dimensions}),
            FieldTypeParserState::TypeTagChar => return Ok(FieldType{scalar_type: ScalarFieldType::Char, dimensions}),
            FieldTypeParserState::TypeTagDouble => return Ok(FieldType{scalar_type: ScalarFieldType::Double, dimensions}),
            FieldTypeParserState::TypeTagFloat => return Ok(FieldType{scalar_type: ScalarFieldType::Float, dimensions}),
            FieldTypeParserState::TypeTagInteger => return Ok(FieldType{scalar_type: ScalarFieldType::Integer, dimensions}),
            FieldTypeParserState::TypeTagLong => return Ok(FieldType{scalar_type: ScalarFieldType::Long, dimensions}),
            FieldTypeParserState::TypeTagShort => return Ok(FieldType{scalar_type: ScalarFieldType::Short, dimensions}),
            FieldTypeParserState::TypeTagBoolean => return Ok(FieldType{scalar_type: ScalarFieldType::Boolean, dimensions}),
            FieldTypeParserState::ClassNameEnd => {
                if class_name.is_empty() {
                    return Err(DemangleError::DemangleFailed);
                }
                let name = BinaryName::new(&class_name)?;
                return Ok(FieldType{scalar_type: ScalarFieldType::Class {name}, dimensions});
            },
            _ => return Err(DemangleError::DemangleFailed),
        }
    }

}

enum FieldTypeParserState {
    Dimensions,
    TypeTagByte,
    TypeTagChar,
    TypeTagDouble,
    TypeTagFloat,
    TypeTagInteger,
    TypeTagLong,
    TypeTagShort,
    TypeTagBoolean,
    TypeTagClass,
    ClassNameEnd,
}

impl fmt::Display for FieldType {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let arrays = if self.dimensions > 0 {
            "[]".repeat(self.dimensions)
        } else {
            "".to_string()
        };
        match self.scalar_type {
            ScalarFieldType::Byte => {
                write!(f, "byte{}", arrays)
            },
            ScalarFieldType::Char => {
                write!(f, "char{}", arrays)
            },
            ScalarFieldType::Double => {
                write!(f, "double{}", arrays)
            },
            ScalarFieldType::Float => {
                write!(f, "float{}", arrays)
            },
            ScalarFieldType::Integer => {
                write!(f, "int{}", arrays)
            },
            ScalarFieldType::Long => {
                write!(f, "long{}", arrays)
            },
            ScalarFieldType::Short => {
                write!(f, "short{}", arrays)
            },
            ScalarFieldType::Boolean => {
                write!(f, "boolean{}", arrays)
            },
            ScalarFieldType::Class{ref name} => {
                write!(f, "{}{}", name, arrays)
            },
        }
    }

}

impl MethodType {

    pub fn new(mangled_method_type: &str) -> Result<MethodType, DemangleError> {
        let mut parameter_types: Vec<FieldType> = Vec::new();
        let return_type: Option<FieldType>;
        let mut dimensions = 0 as usize;
        let mut state = MethodTypeParserState::ParametersStart;
        let mut char_iter = mangled_method_type.chars();
        let mut class_name = String::from("");
        loop {
            let next_char = char_iter.next();
            match next_char {
                Some(c) => {
                    match state {
                        MethodTypeParserState::ParametersStart => {
                            match c {
                                '(' => state = MethodTypeParserState::ParameterDimensions,
                                _ => return Err(DemangleError::DemangleFailed),
                            }
                        },
                        MethodTypeParserState::ParameterDimensions => {
                            match c {
                                '[' => dimensions += 1 as usize,
                                'B' => {
                                    parameter_types.push(FieldType{scalar_type: ScalarFieldType::Byte, dimensions});
                                    dimensions = 0;
                                },
                                'C' => {
                                    parameter_types.push(FieldType{scalar_type: ScalarFieldType::Char, dimensions});
                                    dimensions = 0;
                                },
                                'D' => {
                                    parameter_types.push(FieldType{scalar_type: ScalarFieldType::Double, dimensions});
                                    dimensions = 0;
                                },
                                'F' => {
                                    parameter_types.push(FieldType{scalar_type: ScalarFieldType::Float, dimensions});
                                    dimensions = 0;
                                },
                                'I' => {
                                    parameter_types.push(FieldType{scalar_type: ScalarFieldType::Integer, dimensions});
                                    dimensions = 0;
                                },
                                'J' => {
                                    parameter_types.push(FieldType{scalar_type: ScalarFieldType::Long, dimensions});
                                    dimensions = 0;
                                },
                                'S' => {
                                    parameter_types.push(FieldType{scalar_type: ScalarFieldType::Short, dimensions});
                                    dimensions = 0;
                                },
                                'Z' => {
                                    parameter_types.push(FieldType{scalar_type: ScalarFieldType::Boolean, dimensions});
                                    dimensions = 0;
                                },
                                'L' => state = MethodTypeParserState::ParameterTypeTagClass,
                                ')' => {
                                    if dimensions == 0 {
                                        state = MethodTypeParserState::ReturnTypeDimensions
                                    } else {
                                        return Err(DemangleError::DemangleFailed)
                                    }
                                },
                                _ => return Err(DemangleError::DemangleFailed),
                            }
                        },
                        MethodTypeParserState::ParameterTypeTagClass => {
                            match c {
                                ';' => {
                                    if class_name.is_empty() {
                                        return Err(DemangleError::DemangleFailed);
                                    }
                                    let name = BinaryName::new(&class_name)?;
                                    parameter_types.push(FieldType{scalar_type: ScalarFieldType::Class {name}, dimensions});
                                    dimensions = 0;
                                    class_name = String::from("");
                                    state = MethodTypeParserState::ParameterDimensions;
                                },
                                _ => class_name.push(c),
                            }

                        },
                        MethodTypeParserState::ReturnTypeDimensions => {
                            match c {
                                '[' => dimensions += 1 as usize,
                                'B' => state = MethodTypeParserState::ReturnTypeTagByte,
                                'C' => state = MethodTypeParserState::ReturnTypeTagChar,
                                'D' => state = MethodTypeParserState::ReturnTypeTagDouble,
                                'F' => state = MethodTypeParserState::ReturnTypeTagFloat,
                                'I' => state = MethodTypeParserState::ReturnTypeTagInteger,
                                'J' => state = MethodTypeParserState::ReturnTypeTagLong,
                                'S' => state = MethodTypeParserState::ReturnTypeTagShort,
                                'Z' => state = MethodTypeParserState::ReturnTypeTagBoolean,
                                'L' => state = MethodTypeParserState::ReturnTypeTagClass,
                                'V' => {
                                    if dimensions == 0 {
                                        state = MethodTypeParserState::ReturnTypeTagVoid
                                    } else {
                                        return Err(DemangleError::DemangleFailed)
                                    }
                                },
                                _ => return Err(DemangleError::DemangleFailed),
                            }
                        },
                        MethodTypeParserState::ReturnTypeTagClass => {
                            match c {
                                ';' => state = MethodTypeParserState::ReturnClassNameEnd,
                                _ => class_name.push(c),
                            }

                        },
                        _ => return Err(DemangleError::DemangleFailed),
                    }
                },
                None => break,
            };
        }
        match state {
            MethodTypeParserState::ReturnTypeTagByte => return_type = Some(FieldType{scalar_type: ScalarFieldType::Byte, dimensions}),
            MethodTypeParserState::ReturnTypeTagChar => return_type = Some(FieldType{scalar_type: ScalarFieldType::Char, dimensions}),
            MethodTypeParserState::ReturnTypeTagDouble => return_type = Some(FieldType{scalar_type: ScalarFieldType::Double, dimensions}),
            MethodTypeParserState::ReturnTypeTagFloat => return_type = Some(FieldType{scalar_type: ScalarFieldType::Float, dimensions}),
            MethodTypeParserState::ReturnTypeTagInteger => return_type = Some(FieldType{scalar_type: ScalarFieldType::Integer, dimensions}),
            MethodTypeParserState::ReturnTypeTagLong => return_type = Some(FieldType{scalar_type: ScalarFieldType::Long, dimensions}),
            MethodTypeParserState::ReturnTypeTagShort => return_type = Some(FieldType{scalar_type: ScalarFieldType::Short, dimensions}),
            MethodTypeParserState::ReturnTypeTagBoolean => return_type = Some(FieldType{scalar_type: ScalarFieldType::Boolean, dimensions}),
            MethodTypeParserState::ReturnTypeTagVoid => return_type = None,
            MethodTypeParserState::ReturnClassNameEnd => {
                if class_name.is_empty() {
                    return Err(DemangleError::DemangleFailed);
                }
                let name = BinaryName::new(&class_name)?;
                return_type = Some(FieldType{scalar_type: ScalarFieldType::Class {name}, dimensions});
            },
            _ => return Err(DemangleError::DemangleFailed),
        }
        return Ok(MethodType{parameter_types, return_type});
    }

    pub fn display_as_method_definition<T: fmt::Display>(&self, method_name: &str, class_name: &T) -> String {
        let return_type = match self.return_type {
            Some(ref t) => format!("{}", t),
            None => "void".to_string(),
        };
        let parameter_types: Vec<String> = self.parameter_types.iter().enumerate()
            .map(|(i, t)| format!("{} p{}", t, i)).collect();
        format!("{} {}.{}({})", return_type, class_name, method_name, parameter_types.join(", ")).to_string()
    }

}

enum MethodTypeParserState {
    ParametersStart,
    ParameterDimensions,
    ParameterTypeTagClass,
    ReturnTypeDimensions,
    ReturnTypeTagByte,
    ReturnTypeTagChar,
    ReturnTypeTagDouble,
    ReturnTypeTagFloat,
    ReturnTypeTagInteger,
    ReturnTypeTagLong,
    ReturnTypeTagShort,
    ReturnTypeTagBoolean,
    ReturnTypeTagVoid,
    ReturnTypeTagClass,
    ReturnClassNameEnd,
}

impl fmt::Display for MethodType {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let return_type = match self.return_type {
            Some(ref t) => format!("{}", t),
            None => "void".to_string(),
        };
        let parameter_types: Vec<String> = self.parameter_types.iter().map(|t| format!("{}", t)).collect();
        write!(f, "{} ({})", return_type, parameter_types.join(", "))
    }

}

#[derive(Fail, Debug)]
pub enum DemangleError {
    #[fail(display = "Failed to demangle")]
    DemangleFailed,
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_demangle_binary_name_valid() {
        assert_eq!(super::BinaryName::new("java/lang/Thread").unwrap(),
                   super::BinaryName{packages: vec!["java".to_owned(), "lang".to_owned()], class: "Thread".to_owned()});
        assert_eq!(super::BinaryName::new("java/lang/Threaд").unwrap(),
                   super::BinaryName{packages: vec!["java".to_owned(), "lang".to_owned()], class: "Threaд".to_owned()});
        assert_eq!(super::BinaryName::new("Thread").unwrap(),
                   super::BinaryName{packages: vec![], class: "Thread".to_owned()});
        assert_eq!(super::BinaryName::new("").unwrap(),
                   super::BinaryName{packages: vec![], class: "".to_owned()});
    }

    #[test]
    fn test_demangle_binary_name_invalid() {
        assert!(super::BinaryName::new("java/lang/Thread/").is_err());
        assert!(super::BinaryName::new("/java/lang/Thread").is_err());
        assert!(super::BinaryName::new("/").is_err());
        assert!(super::BinaryName::new("//").is_err());
        assert!(super::BinaryName::new("java//lang/Thread").is_err());
    }

    #[test]
    fn test_demangle_class_type_valid() {
        assert_eq!(super::ClassType::new("Ljava/lang/Thread;").unwrap(),
                   super::ClassType{name: super::BinaryName{packages: vec!["java".to_owned(), "lang".to_owned()], class: "Thread".to_owned()}});
        assert_eq!(super::ClassType::new("Ljava/lang/Threaд;").unwrap(),
                   super::ClassType{name: super::BinaryName{packages: vec!["java".to_owned(), "lang".to_owned()], class: "Threaд".to_owned()}});
        assert_eq!(super::ClassType::new("LThread;").unwrap(),
                   super::ClassType{name: super::BinaryName{packages: vec![], class: "Thread".to_owned()}});
    }

    #[test]
    fn test_demangle_class_type_invalid() {
        assert!(super::ClassType::new("java/lang/Thread").is_err());
        assert!(super::ClassType::new("java/lang/Thread;").is_err());
        assert!(super::ClassType::new("Ljava/lang/Thread").is_err());
        assert!(super::ClassType::new("Ljava/lang/Thread/;").is_err());
        assert!(super::ClassType::new("L/java/lang/Thread;").is_err());
        assert!(super::ClassType::new("").is_err());
        assert!(super::ClassType::new("L;").is_err());
        assert!(super::ClassType::new("L/;").is_err());
        assert!(super::ClassType::new("L//;").is_err());
        assert!(super::ClassType::new("Ljava//lang/Thread;").is_err());
    }

    #[test]
    fn test_demangle_field_type_valid() {
        assert_eq!(super::FieldType::new("B").unwrap(), super::FieldType{ scalar_type: super::ScalarFieldType::Byte, dimensions: 0});
        assert_eq!(super::FieldType::new("C").unwrap(), super::FieldType{ scalar_type: super::ScalarFieldType::Char, dimensions: 0});
        assert_eq!(super::FieldType::new("D").unwrap(), super::FieldType{ scalar_type: super::ScalarFieldType::Double, dimensions: 0});
        assert_eq!(super::FieldType::new("F").unwrap(), super::FieldType{ scalar_type: super::ScalarFieldType::Float, dimensions: 0});
        assert_eq!(super::FieldType::new("I").unwrap(), super::FieldType{ scalar_type: super::ScalarFieldType::Integer, dimensions: 0});
        assert_eq!(super::FieldType::new("J").unwrap(), super::FieldType{ scalar_type: super::ScalarFieldType::Long, dimensions: 0});
        assert_eq!(super::FieldType::new("S").unwrap(), super::FieldType{ scalar_type: super::ScalarFieldType::Short, dimensions: 0});
        assert_eq!(super::FieldType::new("Z").unwrap(), super::FieldType{ scalar_type: super::ScalarFieldType::Boolean, dimensions: 0});
        assert_eq!(super::FieldType::new("[B").unwrap(),
                   super::FieldType{ scalar_type: super::ScalarFieldType::Byte, dimensions: 1});
        assert_eq!(super::FieldType::new("[C").unwrap(),
                   super::FieldType{ scalar_type: super::ScalarFieldType::Char, dimensions: 1});
        assert_eq!(super::FieldType::new("[D").unwrap(),
                   super::FieldType{ scalar_type: super::ScalarFieldType::Double, dimensions: 1});
        assert_eq!(super::FieldType::new("[F").unwrap(),
                   super::FieldType{ scalar_type: super::ScalarFieldType::Float, dimensions: 1});
        assert_eq!(super::FieldType::new("[I").unwrap(),
                   super::FieldType{ scalar_type: super::ScalarFieldType::Integer, dimensions: 1});
        assert_eq!(super::FieldType::new("[J").unwrap(),
                   super::FieldType{ scalar_type: super::ScalarFieldType::Long, dimensions: 1});
        assert_eq!(super::FieldType::new("[S").unwrap(),
                   super::FieldType{ scalar_type: super::ScalarFieldType::Short, dimensions: 1});
        assert_eq!(super::FieldType::new("[Z").unwrap(),
                   super::FieldType{ scalar_type: super::ScalarFieldType::Boolean, dimensions: 1});
        assert_eq!(super::FieldType::new("Ljava/lang/Thread;").unwrap(),
                   super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{packages:
                   vec!["java".to_owned(), "lang".to_owned()], class: "Thread".to_owned()}}, dimensions: 0});
        assert_eq!(super::FieldType::new("[Ljava/lang/Thread;").unwrap(),
                   super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{packages:
                   vec!["java".to_owned(), "lang".to_owned()], class: "Thread".to_owned()}}, dimensions: 1});
        assert_eq!(super::FieldType::new("[[B").unwrap(),
                   super::FieldType{ scalar_type: super::ScalarFieldType::Byte, dimensions: 2});
        assert_eq!(super::FieldType::new("[[C").unwrap(),
                   super::FieldType{ scalar_type: super::ScalarFieldType::Char, dimensions: 2});
        assert_eq!(super::FieldType::new("[[D").unwrap(),
                   super::FieldType{ scalar_type: super::ScalarFieldType::Double, dimensions: 2});
        assert_eq!(super::FieldType::new("[[F").unwrap(),
                   super::FieldType{ scalar_type: super::ScalarFieldType::Float, dimensions: 2});
        assert_eq!(super::FieldType::new("[[I").unwrap(),
                   super::FieldType{ scalar_type: super::ScalarFieldType::Integer, dimensions: 2});
        assert_eq!(super::FieldType::new("[[J").unwrap(),
                   super::FieldType{ scalar_type: super::ScalarFieldType::Long, dimensions: 2});
        assert_eq!(super::FieldType::new("[[S").unwrap(),
                   super::FieldType{ scalar_type: super::ScalarFieldType::Short, dimensions: 2});
        assert_eq!(super::FieldType::new("[[Z").unwrap(),
                   super::FieldType{ scalar_type: super::ScalarFieldType::Boolean, dimensions: 2});
        assert_eq!(super::FieldType::new("[[Ljava/lang/Thread;").unwrap(),
                   super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{packages:
                   vec!["java".to_owned(), "lang".to_owned()], class: "Thread".to_owned()}}, dimensions: 2});
        assert_eq!(super::FieldType::new("[[Ljava/lang/Threaд;").unwrap(),
                   super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{packages:
                   vec!["java".to_owned(), "lang".to_owned()], class: "Threaд".to_owned()}}, dimensions: 2});
    }

    #[test]
    fn test_demangle_field_type_invalid() {
        assert!(super::FieldType::new("").is_err());
        assert!(super::FieldType::new("[").is_err());
        assert!(super::FieldType::new("[[").is_err());
        assert!(super::FieldType::new("M").is_err());
        assert!(super::FieldType::new("[M").is_err());
        assert!(super::FieldType::new("[BB").is_err());
        assert!(super::FieldType::new("BB").is_err());
        assert!(super::FieldType::new("[B[B").is_err());
        assert!(super::FieldType::new("[B[").is_err());
        assert!(super::FieldType::new("[[M").is_err());
        assert!(super::FieldType::new("Lx").is_err());
        assert!(super::FieldType::new("L;").is_err());
        assert!(super::FieldType::new("L/;").is_err());
    }

    #[test]
    fn test_demangle_method_type_valid() {
        assert_eq!(super::MethodType::new("(BCIDFJSZLjava/lang/Thread;)Ljava/lang/Object;").unwrap(),
                   super::MethodType{ parameter_types: vec![super::FieldType{ scalar_type: super::ScalarFieldType::Byte, dimensions: 0},
                                                     super::FieldType{ scalar_type: super::ScalarFieldType::Char, dimensions: 0},
                                                     super::FieldType{ scalar_type: super::ScalarFieldType::Integer, dimensions: 0},
                                                     super::FieldType{ scalar_type: super::ScalarFieldType::Double, dimensions: 0},
                                                     super::FieldType{ scalar_type: super::ScalarFieldType::Float, dimensions: 0},
                                                     super::FieldType{ scalar_type: super::ScalarFieldType::Long, dimensions: 0},
                                                     super::FieldType{ scalar_type: super::ScalarFieldType::Short, dimensions: 0},
                                                     super::FieldType{ scalar_type: super::ScalarFieldType::Boolean, dimensions: 0},
                                                     super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{ packages:
                                                     vec!["java".to_owned(), "lang".to_owned()], class: "Thread".to_owned()}}, dimensions: 0}],
                return_type: Some(super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{ packages:
                vec!["java".to_owned(), "lang".to_owned()], class: "Object".to_owned()}}, dimensions: 0})});
        assert_eq!(super::MethodType::new("(IDLjava/lang/Thread;)Ljava/lang/Object;").unwrap(),
                   super::MethodType{ parameter_types: vec![super::FieldType{ scalar_type: super::ScalarFieldType::Integer, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Double, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{ packages:
                                                            vec!["java".to_owned(), "lang".to_owned()], class: "Thread".to_owned()}}, dimensions: 0}],
                       return_type: Some(super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{ packages:
                       vec!["java".to_owned(), "lang".to_owned()], class: "Object".to_owned()}}, dimensions: 0})});
        assert_eq!(super::MethodType::new("(IDLjava/lang/Threaд;)Ljava/lang/Oбject;").unwrap(),
                   super::MethodType{ parameter_types: vec![super::FieldType{ scalar_type: super::ScalarFieldType::Integer, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Double, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{ packages:
                                                            vec!["java".to_owned(), "lang".to_owned()], class: "Threaд".to_owned()}}, dimensions: 0}],
                       return_type: Some(super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{ packages:
                       vec!["java".to_owned(), "lang".to_owned()], class: "Oбject".to_owned()}}, dimensions: 0})});
        assert_eq!(super::MethodType::new("([[IDLjava/lang/Thread;)Ljava/lang/Object;").unwrap(),
                   super::MethodType{ parameter_types: vec![super::FieldType{ scalar_type: super::ScalarFieldType::Integer, dimensions: 2},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Double, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{ packages:
                                                            vec!["java".to_owned(), "lang".to_owned()], class: "Thread".to_owned()}}, dimensions: 0}],
                       return_type: Some(super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{ packages:
                       vec!["java".to_owned(), "lang".to_owned()], class: "Object".to_owned()}}, dimensions: 0})});
        assert_eq!(super::MethodType::new("(IDLjava/lang/Thread;BZ)Ljava/lang/Object;").unwrap(),
                   super::MethodType{ parameter_types: vec![super::FieldType{ scalar_type: super::ScalarFieldType::Integer, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Double, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{ packages:
                                                            vec!["java".to_owned(), "lang".to_owned()], class: "Thread".to_owned()}}, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Byte, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Boolean, dimensions: 0}],
                       return_type: Some(super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{ packages:
                       vec!["java".to_owned(), "lang".to_owned()], class: "Object".to_owned()}}, dimensions: 0})});
        assert_eq!(super::MethodType::new("(IDLjava/lang/Thread;[B)Ljava/lang/Object;").unwrap(),
                   super::MethodType{ parameter_types: vec![super::FieldType{ scalar_type: super::ScalarFieldType::Integer, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Double, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{ packages:
                                                            vec!["java".to_owned(), "lang".to_owned()], class: "Thread".to_owned()}}, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Byte, dimensions: 1}],
                       return_type: Some(super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{ packages:
                       vec!["java".to_owned(), "lang".to_owned()], class: "Object".to_owned()}}, dimensions: 0})});
        assert_eq!(super::MethodType::new("(ID[[Ljava/lang/Thread;)Ljava/lang/Object;").unwrap(),
                   super::MethodType{ parameter_types: vec![super::FieldType{ scalar_type: super::ScalarFieldType::Integer, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Double, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{ packages:
                                                            vec!["java".to_owned(), "lang".to_owned()], class: "Thread".to_owned()}}, dimensions: 2}],
                       return_type: Some(super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{ packages:
                       vec!["java".to_owned(), "lang".to_owned()], class: "Object".to_owned()}}, dimensions: 0})});
        assert_eq!(super::MethodType::new("(IDLjava/lang/Thread;)[Ljava/lang/Object;").unwrap(),
                   super::MethodType{ parameter_types: vec![super::FieldType{ scalar_type: super::ScalarFieldType::Integer, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Double, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{ packages:
                                                            vec!["java".to_owned(), "lang".to_owned()], class: "Thread".to_owned()}}, dimensions: 0}],
                       return_type: Some(super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{ packages:
                       vec!["java".to_owned(), "lang".to_owned()], class: "Object".to_owned()}}, dimensions: 1})});
        assert_eq!(super::MethodType::new("(IDLjava/lang/Thread;)B").unwrap(),
                   super::MethodType{ parameter_types: vec![super::FieldType{ scalar_type: super::ScalarFieldType::Integer, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Double, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{ packages:
                                                            vec!["java".to_owned(), "lang".to_owned()], class: "Thread".to_owned()}}, dimensions: 0}],
                       return_type: Some(super::FieldType{ scalar_type: super::ScalarFieldType::Byte, dimensions: 0})});
        assert_eq!(super::MethodType::new("(IDLjava/lang/Thread;)[B").unwrap(),
                   super::MethodType{ parameter_types: vec![super::FieldType{ scalar_type: super::ScalarFieldType::Integer, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Double, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{ packages:
                                                            vec!["java".to_owned(), "lang".to_owned()], class: "Thread".to_owned()}}, dimensions: 0}],
                       return_type: Some(super::FieldType{ scalar_type: super::ScalarFieldType::Byte, dimensions: 1})});
        assert_eq!(super::MethodType::new("(IDLjava/lang/Thread;)V").unwrap(),
                   super::MethodType{ parameter_types: vec![super::FieldType{ scalar_type: super::ScalarFieldType::Integer, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Double, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Class {name: super::BinaryName{ packages:
                                                            vec!["java".to_owned(), "lang".to_owned()], class: "Thread".to_owned()}}, dimensions: 0}],
                       return_type: None});
        assert_eq!(super::MethodType::new("(ID)V").unwrap(),
                   super::MethodType{ parameter_types: vec![super::FieldType{ scalar_type: super::ScalarFieldType::Integer, dimensions: 0},
                                                            super::FieldType{ scalar_type: super::ScalarFieldType::Double, dimensions: 0}],
                       return_type: None});
        assert_eq!(super::MethodType::new("()V").unwrap(), super::MethodType{ parameter_types: vec![], return_type: None});
    }

    #[test]
    fn test_demangle_method_type_invalid() {
        assert!(super::MethodType::new("(IDLjava/lang/Thread;)Ljava/lang/Object").is_err());
        assert!(super::MethodType::new("(IDLjava/lang/Thread)Ljava/lang/Object;").is_err());
        assert!(super::MethodType::new("(IDLjava/lang/Thread;)").is_err());
        assert!(super::MethodType::new("(V)Ljava/lang/Object;").is_err());
        assert!(super::MethodType::new("(IDLjava/lang/Thread;)VV").is_err());
        assert!(super::MethodType::new("(IDLjava/lang/Thread;)[V").is_err());
        assert!(super::MethodType::new("IDLjava/lang/Thread;)Ljava/lang/Object").is_err());
        assert!(super::MethodType::new("(").is_err());
        assert!(super::MethodType::new("()").is_err());
        assert!(super::MethodType::new(")").is_err());
        assert!(super::MethodType::new("").is_err());
        assert!(super::MethodType::new("(IDLjava/lang/Thread;").is_err());
        assert!(super::MethodType::new("(IDLjava/lang/Thread;I").is_err());
        assert!(super::MethodType::new("(IDLjava/lang/Thread;VI").is_err());
        assert!(super::MethodType::new("(IDLjava/lang/Thread;)Ljava/lang/Object;B").is_err());
    }

    #[test]
    fn test_display_as_method_definition() {
        assert_eq!(super::MethodType::new("(IDLjava/lang/Thread;)Ljava/lang/Object;").unwrap()
                       .display_as_method_definition("test", &super::BinaryName::new("java/lang/Thread").unwrap()),
            "java.lang.Object java.lang.Thread.test(int p0, double p1, java.lang.Thread p2)");
        assert_eq!(super::MethodType::new("(IDLjava/lang/Thread;)V").unwrap()
                       .display_as_method_definition("test", &super::BinaryName::new("java/lang/Thread").unwrap()),
                   "void java.lang.Thread.test(int p0, double p1, java.lang.Thread p2)");
        assert_eq!(super::MethodType::new("([[I[DLjava/lang/Thread;)[Ljava/lang/Object;").unwrap()
                       .display_as_method_definition("test", &super::BinaryName::new("java/lang/Thread").unwrap()),
                   "java.lang.Object[] java.lang.Thread.test(int[][] p0, double[] p1, java.lang.Thread p2)");
        assert_eq!(super::MethodType::new("(IDLjava/lang/Thread;)Ljava/lang/Object;").unwrap()
                       .display_as_method_definition("test", &super::ClassType::new("Ljava/lang/Thread;").unwrap()),
                   "java.lang.Object java.lang.Thread.test(int p0, double p1, java.lang.Thread p2)");
        assert_eq!(super::MethodType::new("(IDLjava/lang/Thread;)V").unwrap()
                       .display_as_method_definition("test", &super::ClassType::new("Ljava/lang/Thread;").unwrap()),
                   "void java.lang.Thread.test(int p0, double p1, java.lang.Thread p2)");
        assert_eq!(super::MethodType::new("([[I[DLjava/lang/Thread;)[Ljava/lang/Object;").unwrap()
                       .display_as_method_definition("test", &super::ClassType::new("Ljava/lang/Thread;").unwrap()),
                   "java.lang.Object[] java.lang.Thread.test(int[][] p0, double[] p1, java.lang.Thread p2)");
    }

}
