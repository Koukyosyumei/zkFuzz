; ModuleID = './benchmark/sample/iszero_safe.ll'
source_filename = "./benchmark/sample/iszero_safe.circom"

%struct_template_IsZero = type { i128, i128, i128 }

@constraint = internal global i1 false
@constraint.1 = internal global i1 false
@.str.scanf = private constant [5 x i8] c"%lld\00"
@.str.printf = private constant [5 x i8] c"%ld\0A\00"

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

declare i128 @mod_add(i128, i128, i128)

declare i128 @mod_sub(i128, i128, i128)

declare i128 @mod_mul(i128, i128, i128)

declare i128 @mod_div(i128, i128, i128)

define %struct_template_IsZero* @fn_template_build_IsZero() {
entry:
  %malloccall = tail call i8* @malloc(i32 ptrtoint (%struct_template_IsZero* getelementptr (%struct_template_IsZero, %struct_template_IsZero* null, i32 1) to i32))
  %struct_template_IsZero = bitcast i8* %malloccall to %struct_template_IsZero*
  ret %struct_template_IsZero* %struct_template_IsZero
}

declare noalias i8* @malloc(i32)

define void @fn_template_init_IsZero(%struct_template_IsZero* %0) {
entry:
  %initial.in.input = alloca i128, align 8
  %"gep.IsZero|in.input" = getelementptr inbounds %struct_template_IsZero, %struct_template_IsZero* %0, i32 0, i32 0
  %read.in.input = load i128, i128* %"gep.IsZero|in.input", align 4
  store i128 %read.in.input, i128* %initial.in.input, align 4
  %initial.inv.inter = alloca i128, align 8
  %"free.gep.IsZero|inv.inter" = getelementptr %struct_template_IsZero, %struct_template_IsZero* %0, i32 0, i32 1
  %free.read.inv.inter = load i128, i128* %"free.gep.IsZero|inv.inter", align 4
  store i128 %free.read.inv.inter, i128* %initial.inv.inter, align 4
  %initial.out.output = alloca i128, align 8
  br label %body

body:                                             ; preds = %entry
  %read.in.input1 = load i128, i128* %initial.in.input, align 4
  %ne = icmp ne i128 %read.in.input1, 0
  %read.in.input2 = load i128, i128* %initial.in.input, align 4
  %mod_div = call i128 @mod_div(i128 1, i128 %read.in.input2, i128 9938766679346745377)
  %utils_switch = call i128 @fn_intrinsic_utils_switch(i1 %ne, i128 %mod_div, i128 0)
  %read.in.input3 = load i128, i128* %initial.in.input, align 4
  %mod_sub = call i128 @mod_sub(i128 0, i128 %read.in.input3, i128 9938766679346745377)
  %read.inv.inter = load i128, i128* %initial.inv.inter, align 4
  %mod_mul = call i128 @mod_mul(i128 %mod_sub, i128 %read.inv.inter, i128 9938766679346745377)
  %mod_add = call i128 @mod_add(i128 %mod_mul, i128 1, i128 9938766679346745377)
  %read.out.output = load i128, i128* %initial.out.output, align 4
  call void @fn_intrinsic_utils_constraint(i128 %read.out.output, i128 %mod_add, i1* @constraint)
  store i128 %mod_add, i128* %initial.out.output, align 4
  %read.in.input4 = load i128, i128* %initial.in.input, align 4
  %read.out.output5 = load i128, i128* %initial.out.output, align 4
  %mod_mul6 = call i128 @mod_mul(i128 %read.in.input4, i128 %read.out.output5, i128 9938766679346745377)
  call void @fn_intrinsic_utils_constraint(i128 %mod_mul6, i128 0, i1* @constraint.1)
  br label %exit

exit:                                             ; preds = %body
  %read.inv.inter7 = load i128, i128* %initial.inv.inter, align 4
  %"gep.IsZero|inv.inter" = getelementptr inbounds %struct_template_IsZero, %struct_template_IsZero* %0, i32 0, i32 1
  store i128 %read.inv.inter7, i128* %"gep.IsZero|inv.inter", align 4
  %read.out.output8 = load i128, i128* %initial.out.output, align 4
  %"gep.IsZero|out.output" = getelementptr inbounds %struct_template_IsZero, %struct_template_IsZero* %0, i32 0, i32 2
  store i128 %read.out.output8, i128* %"gep.IsZero|out.output", align 4
  ret void
}

declare i32 @printf(i8*, ...)

declare i32 @scanf(i8*, ...)

define i32 @main() {
entry:
  %instance = call %struct_template_IsZero* @fn_template_build_IsZero()
  %"gep.IsZero|in.input" = getelementptr %struct_template_IsZero, %struct_template_IsZero* %instance, i32 0, i32 0
  %0 = call i32 (i8*, ...) @scanf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.scanf, i32 0, i32 0), i128* %"gep.IsZero|in.input")
  %"gep.IsZero|inv.inter" = getelementptr %struct_template_IsZero, %struct_template_IsZero* %instance, i32 0, i32 1
  %1 = call i32 (i8*, ...) @scanf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.scanf, i32 0, i32 0), i128* %"gep.IsZero|inv.inter")
  call void @fn_template_init_IsZero(%struct_template_IsZero* %instance)
  %"gep.IsZero|out.output" = getelementptr %struct_template_IsZero, %struct_template_IsZero* %instance, i32 0, i32 2
  %"val.gep.IsZero|out.output" = load i128, i128* %"gep.IsZero|out.output", align 4
  %2 = trunc i128 %"val.gep.IsZero|out.output" to i64
  %3 = lshr i128 %"val.gep.IsZero|out.output", 64
  %4 = trunc i128 %3 to i64
  %5 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.printf, i32 0, i32 0), i64 %4)
  %6 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.printf, i32 0, i32 0), i64 %2)
  ret i32 0
}

attributes #0 = { nofree nosync nounwind readnone speculatable willreturn }
