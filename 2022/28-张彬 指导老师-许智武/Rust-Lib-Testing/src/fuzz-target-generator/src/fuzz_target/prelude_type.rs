//To deal with some prelude type
use crate::fuzz_target::api_util;
use crate::fuzz_target::call_type::CallType;
use crate::fuzz_target::impl_util::FullNameMap;
use crate::fuzz_target::fuzzable_type::FuzzableType;
pub use lazy_static::*;
use rustdoc::clean::{self};
use std::collections::{HashMap, HashSet};

lazy_static! {
    static ref PRELUDED_TYPE: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("core::option::Option", "Option");
        m.insert("core::result::Result", "Result");
        m.insert("alloc::string::String", "String");
        //m.insert("alloc::boxed::Box", "Box");
        m
    };
}

lazy_static! {
    static ref PRELUDED_STRUCT: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("alloc::string::String", "String");
        m
    };
}

// lazy_static! {
//     static ref PRELUDED_TYPE_TRAIT: HashMap<&'static str, Vec<&'static str>> = {
//         let mut m = HashMap::new();
//         m.insert("String", vec!["core::fmt::Display", "core::fmt::Debug"]);
//         //m.insert("alloc::boxed::Box", "Box");
//         m
//     };
// }

// pub fn find_prelude_types_by_trait_bound(trait_bounds: &Vec<String>) -> Vec<FuzzableType> {
//     let mut res_types = Vec::new();
//     for type_ in PRELUDED_TYPE_TRAIT.keys() {
//         let traits_ = PRELUDED_TYPE_TRAIT.get(*type_).unwrap();
//         let minusion = trait_bounds.iter().filter(|&u| !traits_.contains(&u.as_str())).collect::<Vec<_>>();
//         if minusion.len() == 0 {
//             match *type_ {
//                 "String" => {
//                     if !res_types.contains(&FuzzableType::String) {
//                         res_types.push(FuzzableType::String);
//                     }
//                 }
//                 _ => { }
//             }
//         }
//     }
//     res_types
// }

pub fn is_preluded_struct(type_name: &String) -> bool {
    if PRELUDED_STRUCT.contains_key(type_name.as_str()) {
        return true;
    } else {
        return false;
    }
}

static _OPTION: &'static str = "Option";
static _RESULT: &'static str = "Result";
static _STRING: &'static str = "String";

pub fn is_preluded_type(type_name: &String) -> bool {
    if PRELUDED_TYPE.contains_key(type_name.as_str()) {
        println!("Preluded type: {:?}", type_name);
        return true;
    } else {
        return false;
    }
}

pub fn get_all_preluded_type() -> HashSet<String> {
    let mut res = HashSet::new();
    for (prelude_type_, _) in PRELUDED_TYPE.iter() {
        res.insert(prelude_type_.to_string());
    }
    res
}

pub fn preluded_type(type_: &clean::Type, full_name_map: &FullNameMap) -> bool {
    let def_id = type_.def_id_no_primitives().unwrap(); //type_.def_id().unwrap();
    if let Some(type_name) = full_name_map.get_full_name(&def_id) {
        println!("type name: {}", type_name);
        if is_preluded_type(&type_name) {
            return true;
        }
    }
    return false;
}

pub fn to_strip_type_name(type_name: &String) -> String {
    if PRELUDED_TYPE.contains_key(type_name.as_str()) {
        PRELUDED_TYPE.get(type_name.as_str()).unwrap().to_string()
    } else {
        type_name.clone()
    }
}

// TODO:目前只考虑引用、裸指针的情况，元组，切片，数组都暂时不考虑
// 暂时只考虑Result和Option
// TODO:Box,...
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PreludeType {
    NotPrelude(clean::Type),
    PreludeOption(clean::Type),
    PreludeResult {
        ok_type: clean::Type,
        err_type: clean::Type,
    },
    PreludeString(clean::Type),
}

impl PreludeType {
    pub fn from_type(type_: &clean::Type, full_name_map: &FullNameMap) -> Self {
        match type_ {
            clean::Type::Path { path, .. } => {
                // println!("is preluded type");
                if path.segments.len() != 1 {
                    // println!("from type unexpected type: {:#?}", type_);
                    return PreludeType::NotPrelude(type_.clone());
                }
                let segment = &path.segments[0];
                if _OPTION == segment.name.as_str() {
                    extract_option(path, type_)
                } else if _RESULT == segment.name.as_str() {
                    extract_result(path, type_)
                } else if _STRING == segment.name.as_str() {
                    extract_string(path, type_)
                } else {
                    // println!("other prelude type, {}", segment.name.as_str());
                    PreludeType::NotPrelude(type_.clone())
                }
                // if preluded_type(type_, full_name_map) {
                //     println!("is preluded type");
                //     let def_id = type_.def_id_no_primitives().unwrap(); // type_.def_id().unwrap();
                //     let type_full_name = full_name_map.get_full_name(&def_id).unwrap();
                //     let strip_type_name_string = to_strip_type_name(&type_full_name);
                //     let strip_type_name = strip_type_name_string.as_str();
                //     if _OPTION == strip_type_name {
                //         extract_option(path, type_)
                //     } else if _RESULT == strip_type_name {
                //         extract_result(path, type_)
                //     } else if _STRING == strip_type_name {
                //         extract_string(path, type_)
                //     } else {
                //         //println!("other prelude type");
                //         PreludeType::NotPrelude(type_.clone())
                //     }
                // } else {
                //     println!("no preluded type");
                //     println!("{:#?}", type_);
                //     PreludeType::NotPrelude(type_.clone())
                // }
            }
            _ => PreludeType::NotPrelude(type_.clone()),
        }
    }

    pub fn _to_type_name(&self, full_name_map: &FullNameMap) -> String {
        match self {
            PreludeType::NotPrelude(type_) => api_util::_type_name(type_, full_name_map),
            PreludeType::PreludeOption(type_) => {
                let inner_type_name = api_util::_type_name(type_, full_name_map);
                format!("Option<{}>", inner_type_name)
            }
            PreludeType::PreludeResult { ok_type, err_type } => {
                let ok_type_name = api_util::_type_name(ok_type, full_name_map);
                let err_type_name = api_util::_type_name(err_type, full_name_map);
                format!("Result<{}, {}>", ok_type_name, err_type_name)
            }
            PreludeType::PreludeString(type_) => api_util::_type_name(type_, full_name_map)
        }
    }

    pub fn _is_final_type(&self) -> bool {
        match self {
            PreludeType::NotPrelude(..) | PreludeType::PreludeString(..) => true,
            PreludeType::PreludeResult { .. } | PreludeType::PreludeOption(..) => false,
        }
    }

    pub fn _get_final_type(&self) -> clean::Type {
        //获得最终的类型
        match self {
            PreludeType::NotPrelude(type_) => type_.clone(),
            PreludeType::PreludeOption(type_) => type_.clone(),
            PreludeType::PreludeResult { ok_type, .. } => {
                //Result只取ok的那部分
                ok_type.clone()
            },
            PreludeType::PreludeString(type_) => type_.clone(),
        }
    }

    // How to get final type
    pub fn _unwrap_call_type(&self, inner_call_type: &CallType) -> CallType {
        match self {
            PreludeType::NotPrelude(..) => inner_call_type.clone(),
            PreludeType::PreludeOption(_type_) => {
                CallType::_UnwrapOption(Box::new(inner_call_type.clone()))
            }
            PreludeType::PreludeResult { .. } => {
                CallType::_UnwrapResult(Box::new(inner_call_type.clone()))
            }
            PreludeType::PreludeString(..) => inner_call_type.clone(),
        }
    }

    pub fn _to_call_type(&self, inner_call_type: &CallType) -> CallType {
        match self {
            PreludeType::NotPrelude(..) => inner_call_type.clone(),
            PreludeType::PreludeOption(..) => {
                CallType::_ToOption(Box::new(inner_call_type.clone()))
            }
            PreludeType::PreludeResult { .. } => {
                CallType::_ToResult(Box::new(inner_call_type.clone()))
            }
            PreludeType::PreludeString { .. } => {
                // TODO: CallType for String
                inner_call_type.clone()
            }
        }
    }
}

fn extract_option(path: &clean::Path, type_: &clean::Type) -> PreludeType {
    // println!("{:#?}", type_);
    let segments = &path.segments;
    for path_segment in segments {
        let generic_args = &path_segment.args;
        match generic_args {
            clean::GenericArgs::AngleBracketed { args, .. } => {
                if args.len() != 1 {
                    continue;
                }
                let arg = &args[0];
                if let clean::GenericArg::Type(type_) = arg {
                    return PreludeType::PreludeOption(type_.clone());
                }
            }
            clean::GenericArgs::Parenthesized { .. } => {}
        }
    }
    return PreludeType::NotPrelude(type_.clone());
}

fn extract_result(path: &clean::Path, type_: &clean::Type) -> PreludeType {
    let segments = &path.segments;
    for path_segment in segments {
        let generic_args = &path_segment.args;
        match generic_args {
            clean::GenericArgs::AngleBracketed { args, .. } => {
                if args.len() != 2 {
                    continue;
                }
                let ok_arg = &args[0];
                let err_arg = &args[1];
                if let clean::GenericArg::Type(ok_type_) = ok_arg {
                    if let clean::GenericArg::Type(err_type_) = err_arg {
                        return PreludeType::PreludeResult {
                            ok_type: ok_type_.clone(),
                            err_type: err_type_.clone(),
                        };
                    }
                }
            }
            clean::GenericArgs::Parenthesized { .. } => {}
        }
    }
    return PreludeType::NotPrelude(type_.clone());
}

fn extract_string(path: &clean::Path, type_: &clean::Type) -> PreludeType {
    PreludeType::PreludeString(type_.clone())
}

pub fn _prelude_type_need_special_dealing(
    type_: &clean::Type,
    full_name_map: &FullNameMap,
) -> bool {
    // let prelude_type = PreludeType::from_type(type_, full_name_map);
    // let final_type = prelude_type._get_final_type();
    // if final_type == *type_ {
    //     false
    // } else {
    //     true
    // }
    // println!("prelude type result: {:#?}", PreludeType::from_type(type_, full_name_map));
    match PreludeType::from_type(type_, full_name_map) {
        PreludeType::NotPrelude(..) => false,
        PreludeType::PreludeOption(..) => true,
        PreludeType::PreludeResult{..} => true,
        PreludeType::PreludeString(..) => false,
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum _PreludeHelper {
    _ResultHelper,
    _OptionHelper,
}

impl _PreludeHelper {
    pub fn _from_call_type(call_type: &CallType) -> HashSet<_PreludeHelper> {
        match call_type {
            CallType::_DirectCall 
            | CallType::_NotCompatible 
            | CallType::_AsConvert(_) 
            | CallType::_ToString
            | CallType::_Debug
            | CallType::_Display 
            | CallType::_GenericCall => {
                HashSet::new()
            }
            CallType::_BorrowedRef(inner_call_type)
            | CallType::_ConstRawPointer(inner_call_type, _)
            | CallType::_MutBorrowedRef(inner_call_type)
            | CallType::_MutRawPointer(inner_call_type, _)
            | CallType::_Deref(inner_call_type)
            | CallType::_ToOption(inner_call_type)
            | CallType::_ToResult(inner_call_type)
            | CallType::_UnsafeDeref(inner_call_type) => {
                _PreludeHelper::_from_call_type(&**inner_call_type)
            }
            CallType::_UnwrapOption(inner_call_type) => {
                let mut inner_helpers = _PreludeHelper::_from_call_type(inner_call_type);
                inner_helpers.insert(_PreludeHelper::_OptionHelper);
                inner_helpers
            }
            CallType::_UnwrapResult(inner_call_type) => {
                let mut inner_helpers = _PreludeHelper::_from_call_type(inner_call_type);
                inner_helpers.insert(_PreludeHelper::_ResultHelper);
                inner_helpers
            }
        }
    }

    pub fn _to_helper_function(&self) -> &'static str {
        match self {
            _PreludeHelper::_ResultHelper => _unwrap_result_function(),
            _PreludeHelper::_OptionHelper => _unwrap_option_function(),
        }
    }
}

fn _unwrap_result_function() -> &'static str {
    "fn _unwrap_result<T, E>(_res: Result<T, E>) -> T {
    match _res {
        Ok(_t) => _t,
        Err(_) => {
            use std::process;
            process::exit(0);
        },
    }
}\n"
}

fn _unwrap_option_function() -> &'static str {
    "fn _unwrap_option<T>(_opt: Option<T>) -> T {
    match _opt {
        Some(_t) => _t,
        None => {
            use std::process;
            process::exit(0);
        }
    }
}\n"
}
