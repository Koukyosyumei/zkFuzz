use inkwell::{
    types::{AnyTypeEnum, BasicType, BasicTypeEnum},
    values::{BasicValueEnum, AnyValue},
    AddressSpace,
};

fn valid_used_type(ty: &BasicTypeEnum) -> bool {
    match ty {
        BasicTypeEnum::IntType(..) => true,
        BasicTypeEnum::PointerType(ptr_ty) => {
            let ptr_ele_ty = ptr_ty.get_element_type();
            ptr_ele_ty.is_array_type() || ptr_ele_ty.is_struct_type()
        }
        _ => false,
    }
}

pub fn check_used_value(val: &BasicValueEnum) {
    let res = valid_used_type(&val.get_type());
    if !res {
        println!("Error: Invalid used value: {}", val.print_to_string());
        unreachable!();
    }
}

pub fn check_used_type(ty: &BasicTypeEnum) {
    let res = valid_used_type(ty);
    if !res {
        println!("Error: Invalid used type: {}", ty.print_to_string());
        unreachable!();
    }
}

pub fn wrap_type2used<'ctx>(ty: &BasicTypeEnum<'ctx>) -> BasicTypeEnum<'ctx> {
    match ty {
        BasicTypeEnum::ArrayType(arr_ty) => {
            return arr_ty.ptr_type(AddressSpace::default()).as_basic_type_enum();
        }
        BasicTypeEnum::StructType(strt_ty) => {
            return strt_ty.ptr_type(AddressSpace::default()).as_basic_type_enum();
        }
        _ => {
            return ty.clone();
        }
    }
}

pub fn unwrap_used_type<'ctx>(ty: &BasicTypeEnum<'ctx>) -> BasicTypeEnum<'ctx> {
    match ty {
        BasicTypeEnum::PointerType(ptr_ty) => {
            let ele_ty = ptr_ty.get_element_type();
            match ele_ty {
                AnyTypeEnum::ArrayType(arr_ty) => arr_ty.as_basic_type_enum(),
                AnyTypeEnum::StructType(strt_ty) => strt_ty.as_basic_type_enum(),
                AnyTypeEnum::IntType(int_ty) => int_ty.as_basic_type_enum(),
                _ => {
                    println!("Error: Unknown type: {}", ele_ty.print_to_string());
                    unreachable!();
                }
            }
        }
        _ => {
            return ty.clone();
        }
    }
}
