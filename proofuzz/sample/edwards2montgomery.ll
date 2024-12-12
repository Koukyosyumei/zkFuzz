; ModuleID = 'edwards2montgomery.circom'
source_filename = "../../benchmark/sample/edwards2montgomery.circom"

%struct_template_Edwards2Montgomery = type { [256 x i128]*, [256 x i128]* }

@constraint = external global i1
@constraint.1 = external global i1

define void @fn_intrinsic_utils_constraint(i128 %0, i128 %1, i1* %2) {
entry:
  %constraint = icmp eq i128 %0, %1
  store i1 %constraint, i1* %2, align 1
  ret void
}

define void @fn_intrinsic_utils_constraint_array([256 x i128]* %0, [256 x i128]* %1, i1* %2) {
entry:
  ret void
}

define i128 @fn_intrinsic_utils_switch(i1 %0, i128 %1, i128 %2) {
entry:
  br i1 %0, label %if.true, label %if.false

if.true:                                          ; preds = %entry
  ret i128 %1

if.false:                                         ; preds = %entry
  ret i128 %2
}

; Function Attrs: nofree nosync nounwind readnone speculatable willreturn
declare fp128 @llvm.powi.f128.i32(fp128, i32) #0

define i128 @fn_intrinsic_utils_powi(i128 %0, i128 %1) {
entry:
  %utils_powi.base = uitofp i128 %0 to fp128
  %utils_powi.power = trunc i128 %1 to i32
  %utils_powi.cal = call fp128 @llvm.powi.f128.i32(fp128 %utils_powi.base, i32 %utils_powi.power)
  %utils_powi.ret = fptoui fp128 %utils_powi.cal to i128
  ret i128 %utils_powi.ret
}

define i128 @fn_intrinsic_utils_init() {
entry:
  ret i128 0
}

define void @fn_intrinsic_utils_assert(i1 %0) {
entry:
  ret void
}

define void @fn_intrinsic_utils_arraydim(i128* %0, ...) {
entry:
  ret void
}

define %struct_template_Edwards2Montgomery* @fn_template_build_Edwards2Montgomery() {
entry:
  %malloccall = tail call i8* @malloc(i32 ptrtoint (%struct_template_Edwards2Montgomery* getelementptr (%struct_template_Edwards2Montgomery, %struct_template_Edwards2Montgomery* null, i32 1) to i32))
  %struct_template_Edwards2Montgomery = bitcast i8* %malloccall to %struct_template_Edwards2Montgomery*
  ret %struct_template_Edwards2Montgomery* %struct_template_Edwards2Montgomery
}

declare noalias i8* @malloc(i32)

define void @fn_template_init_Edwards2Montgomery(%struct_template_Edwards2Montgomery* %0) {
entry:
  %initial.in.input = alloca [256 x i128]*, align 8
  %malloccall = tail call i8* @malloc(i32 ptrtoint ([256 x i128]* getelementptr ([256 x i128], [256 x i128]* null, i32 1) to i32))
  %Edwards2Montgomeryinlinearray = bitcast i8* %malloccall to [256 x i128]*
  store [256 x i128]* %Edwards2Montgomeryinlinearray, [256 x i128]** %initial.in.input, align 8
  %"gep.Edwards2Montgomery|in.input" = getelementptr inbounds %struct_template_Edwards2Montgomery, %struct_template_Edwards2Montgomery* %0, i32 0, i32 0
  %read.in.input = load [256 x i128]*, [256 x i128]** %"gep.Edwards2Montgomery|in.input", align 8
  store [256 x i128]* %read.in.input, [256 x i128]** %initial.in.input, align 8
  %initial.out.output = alloca [256 x i128]*, align 8
  %malloccall1 = tail call i8* @malloc(i32 ptrtoint ([256 x i128]* getelementptr ([256 x i128], [256 x i128]* null, i32 1) to i32))
  %Edwards2Montgomeryinlinearray2 = bitcast i8* %malloccall1 to [256 x i128]*
  store [256 x i128]* %Edwards2Montgomeryinlinearray2, [256 x i128]** %initial.out.output, align 8
  br label %body

body:                                             ; preds = %entry
  %var_getter = load [256 x i128]*, [256 x i128]** %initial.in.input, align 8
  %array_getter = getelementptr inbounds [256 x i128], [256 x i128]* %var_getter, i128 0, i128 1
  %read.in.input3 = load i128, i128* %array_getter, align 4
  %add = add i128 1, %read.in.input3
  %var_getter4 = load [256 x i128]*, [256 x i128]** %initial.in.input, align 8
  %array_getter5 = getelementptr inbounds [256 x i128], [256 x i128]* %var_getter4, i128 0, i128 1
  %read.in.input6 = load i128, i128* %array_getter5, align 4
  %sub = sub i128 1, %read.in.input6
  %sdiv = sdiv i128 %add, %sub
  %var_getter7 = load [256 x i128]*, [256 x i128]** %initial.out.output, align 8
  %write.out.output = getelementptr inbounds [256 x i128], [256 x i128]* %var_getter7, i128 0, i128 0
  store i128 %sdiv, i128* %write.out.output, align 4
  %var_getter8 = load [256 x i128]*, [256 x i128]** %initial.out.output, align 8
  %array_getter9 = getelementptr inbounds [256 x i128], [256 x i128]* %var_getter8, i128 0, i128 0
  %read.out.output = load i128, i128* %array_getter9, align 4
  %var_getter10 = load [256 x i128]*, [256 x i128]** %initial.in.input, align 8
  %array_getter11 = getelementptr inbounds [256 x i128], [256 x i128]* %var_getter10, i128 0, i128 0
  %read.in.input12 = load i128, i128* %array_getter11, align 4
  %sdiv13 = sdiv i128 %read.out.output, %read.in.input12
  %var_getter14 = load [256 x i128]*, [256 x i128]** %initial.out.output, align 8
  %write.out.output15 = getelementptr inbounds [256 x i128], [256 x i128]* %var_getter14, i128 0, i128 1
  store i128 %sdiv13, i128* %write.out.output15, align 4
  %var_getter16 = load [256 x i128]*, [256 x i128]** %initial.out.output, align 8
  %array_getter17 = getelementptr inbounds [256 x i128], [256 x i128]* %var_getter16, i128 0, i128 0
  %read.out.output18 = load i128, i128* %array_getter17, align 4
  %var_getter19 = load [256 x i128]*, [256 x i128]** %initial.in.input, align 8
  %array_getter20 = getelementptr inbounds [256 x i128], [256 x i128]* %var_getter19, i128 0, i128 1
  %read.in.input21 = load i128, i128* %array_getter20, align 4
  %sub22 = sub i128 1, %read.in.input21
  %mul = mul i128 %read.out.output18, %sub22
  %var_getter23 = load [256 x i128]*, [256 x i128]** %initial.in.input, align 8
  %array_getter24 = getelementptr inbounds [256 x i128], [256 x i128]* %var_getter23, i128 0, i128 1
  %read.in.input25 = load i128, i128* %array_getter24, align 4
  %add26 = add i128 1, %read.in.input25
  call void @fn_intrinsic_utils_constraint(i128 %mul, i128 %add26, i1* @constraint)
  %var_getter27 = load [256 x i128]*, [256 x i128]** %initial.out.output, align 8
  %array_getter28 = getelementptr inbounds [256 x i128], [256 x i128]* %var_getter27, i128 0, i128 1
  %read.out.output29 = load i128, i128* %array_getter28, align 4
  %var_getter30 = load [256 x i128]*, [256 x i128]** %initial.in.input, align 8
  %array_getter31 = getelementptr inbounds [256 x i128], [256 x i128]* %var_getter30, i128 0, i128 0
  %read.in.input32 = load i128, i128* %array_getter31, align 4
  %mul33 = mul i128 %read.out.output29, %read.in.input32
  %var_getter34 = load [256 x i128]*, [256 x i128]** %initial.out.output, align 8
  %array_getter35 = getelementptr inbounds [256 x i128], [256 x i128]* %var_getter34, i128 0, i128 0
  %read.out.output36 = load i128, i128* %array_getter35, align 4
  call void @fn_intrinsic_utils_constraint(i128 %mul33, i128 %read.out.output36, i1* @constraint.1)
  br label %exit

exit:                                             ; preds = %body
  %ptr_cast = bitcast [256 x i128]** %initial.in.input to i128*
  call void (i128*, ...) @fn_intrinsic_utils_arraydim(i128* %ptr_cast, i128 2)
  %ptr_cast37 = bitcast [256 x i128]** %initial.out.output to i128*
  call void (i128*, ...) @fn_intrinsic_utils_arraydim(i128* %ptr_cast37, i128 2)
  %read.out.output38 = load [256 x i128]*, [256 x i128]** %initial.out.output, align 8
  %"gep.Edwards2Montgomery|out.output" = getelementptr inbounds %struct_template_Edwards2Montgomery, %struct_template_Edwards2Montgomery* %0, i32 0, i32 1
  store [256 x i128]* %read.out.output38, [256 x i128]** %"gep.Edwards2Montgomery|out.output", align 8
  ret void
}

attributes #0 = { nofree nosync nounwind readnone speculatable willreturn }
