; ModuleID = './benchmark/sample/iszero_vuln.ll'
source_filename = "./benchmark/sample/iszero_vuln.circom"

%struct_template_IsZero = type { i128, i128, i128 }

@constraint = internal global i1 false
@.str.scanf = private constant [5 x i8] c"%lld\00"
@.str.1bit.printf = private constant [4 x i8] c"%d\0A\00"
@.str.printf = private constant [5 x i8] c"%ld\0A\00"
@0 = private unnamed_addr constant [61 x i8] c"Error: Under-Constraint-Condition met. Terminating program.\0A\00", align 1

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
  %initial.out.output = alloca i128, align 8
  br label %body

body:                                             ; preds = %entry
  %read.in.input1 = load i128, i128* %initial.in.input, align 4
  %ne = icmp ne i128 %read.in.input1, 0
  %read.in.input2 = load i128, i128* %initial.in.input, align 4
  %mod_div = call i128 @mod_div(i128 1, i128 %read.in.input2, i128 9938766679346745377)
  %utils_switch = call i128 @fn_intrinsic_utils_switch(i1 %ne, i128 %mod_div, i128 0)
  store i128 %utils_switch, i128* %initial.inv.inter, align 4
  %read.in.input3 = load i128, i128* %initial.in.input, align 4
  %mod_sub = call i128 @mod_sub(i128 0, i128 %read.in.input3, i128 9938766679346745377)
  %read.inv.inter = load i128, i128* %initial.inv.inter, align 4
  %mod_mul = call i128 @mod_mul(i128 %mod_sub, i128 %read.inv.inter, i128 9938766679346745377)
  %mod_add = call i128 @mod_add(i128 %mod_mul, i128 1, i128 9938766679346745377)
  store i128 %mod_add, i128* %initial.out.output, align 4
  %read.out.output = load i128, i128* %initial.out.output, align 4
  call void @fn_intrinsic_utils_constraint(i128 %read.out.output, i128 %mod_add, i1* @constraint)
  br label %exit

exit:                                             ; preds = %body
  %read.inv.inter4 = load i128, i128* %initial.inv.inter, align 4
  %"gep.IsZero|inv.inter" = getelementptr inbounds %struct_template_IsZero, %struct_template_IsZero* %0, i32 0, i32 1
  store i128 %read.inv.inter4, i128* %"gep.IsZero|inv.inter", align 4
  %read.out.output5 = load i128, i128* %initial.out.output, align 4
  %"gep.IsZero|out.output" = getelementptr inbounds %struct_template_IsZero, %struct_template_IsZero* %0, i32 0, i32 2
  store i128 %read.out.output5, i128* %"gep.IsZero|out.output", align 4
  ret void
}

declare i32 @printf(i8*, ...)

declare i32 @scanf(i8*, ...)

declare void @exit(i32)

define void @cloned_fn_template_init_IsZero(%struct_template_IsZero* %0) {
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
  store i128 %mod_add, i128* %initial.out.output, align 4
  %read.out.output = load i128, i128* %initial.out.output, align 4
  call void @fn_intrinsic_utils_constraint(i128 %read.out.output, i128 %mod_add, i1* @constraint)
  br label %exit

exit:                                             ; preds = %body
  %read.inv.inter4 = load i128, i128* %initial.inv.inter, align 4
  %"gep.IsZero|inv.inter" = getelementptr inbounds %struct_template_IsZero, %struct_template_IsZero* %0, i32 0, i32 1
  store i128 %read.inv.inter4, i128* %"gep.IsZero|inv.inter", align 4
  %read.out.output5 = load i128, i128* %initial.out.output, align 4
  %"gep.IsZero|out.output" = getelementptr inbounds %struct_template_IsZero, %struct_template_IsZero* %0, i32 0, i32 2
  store i128 %read.out.output5, i128* %"gep.IsZero|out.output", align 4
  ret void
}

define i32 @main() {
entry:
  %instance = call %struct_template_IsZero* @fn_template_build_IsZero()
  %"gep.IsZero|in.input" = getelementptr %struct_template_IsZero, %struct_template_IsZero* %instance, i32 0, i32 0
  %0 = alloca i64, align 8
  %1 = alloca i64, align 8
  %2 = call i32 (i8*, ...) @scanf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.scanf, i32 0, i32 0), i64* %0)
  %3 = call i32 (i8*, ...) @scanf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.scanf, i32 0, i32 0), i64* %1)
  %4 = load i64, i64* %0, align 4
  %5 = load i64, i64* %1, align 4
  %6 = zext i64 %4 to i128
  %7 = zext i64 %5 to i128
  %8 = shl i128 %7, 64
  %9 = or i128 %6, %8
  store i128 %9, i128* %"gep.IsZero|in.input", align 4
  %"gep.IsZero|inv.inter" = getelementptr %struct_template_IsZero, %struct_template_IsZero* %instance, i32 0, i32 1
  %10 = alloca i64, align 8
  %11 = alloca i64, align 8
  %12 = call i32 (i8*, ...) @scanf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.scanf, i32 0, i32 0), i64* %10)
  %13 = call i32 (i8*, ...) @scanf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.scanf, i32 0, i32 0), i64* %11)
  %14 = load i64, i64* %10, align 4
  %15 = load i64, i64* %11, align 4
  %16 = zext i64 %14 to i128
  %17 = zext i64 %15 to i128
  %18 = shl i128 %17, 64
  %19 = or i128 %16, %18
  store i128 %19, i128* %"gep.IsZero|inv.inter", align 4
  %is_cloned_satisfy_constraints = alloca i1, align 1
  call void @cloned_fn_template_init_IsZero(%struct_template_IsZero* %instance)
  %"gep.IsZero|out.output" = getelementptr %struct_template_IsZero, %struct_template_IsZero* %instance, i32 0, i32 2
  %"cloned_result.gep.IsZero|out.output" = load i128, i128* %"gep.IsZero|out.output", align 4
  %20 = trunc i128 %"cloned_result.gep.IsZero|out.output" to i64
  %21 = lshr i128 %"cloned_result.gep.IsZero|out.output", 64
  %22 = trunc i128 %21 to i64
  %23 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.printf, i32 0, i32 0), i64 %22)
  %24 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.printf, i32 0, i32 0), i64 %20)
  store i1 true, i1* %is_cloned_satisfy_constraints, align 1
  %25 = load i1, i1* @constraint, align 1
  %26 = load i1, i1* %is_cloned_satisfy_constraints, align 1
  %27 = and i1 %26, %25
  store i1 %27, i1* %is_cloned_satisfy_constraints, align 1
  %28 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @.str.1bit.printf, i32 0, i32 0), i1 %27)
  %is_original_satisfy_constraints = alloca i1, align 1
  call void @fn_template_init_IsZero(%struct_template_IsZero* %instance)
  %"gep.IsZero|out.output1" = getelementptr %struct_template_IsZero, %struct_template_IsZero* %instance, i32 0, i32 2
  %"original_result.gep.IsZero|out.output" = load i128, i128* %"gep.IsZero|out.output1", align 4
  %29 = trunc i128 %"original_result.gep.IsZero|out.output" to i64
  %30 = lshr i128 %"original_result.gep.IsZero|out.output", 64
  %31 = trunc i128 %30 to i64
  %32 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.printf, i32 0, i32 0), i64 %31)
  %33 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.printf, i32 0, i32 0), i64 %29)
  store i1 true, i1* %is_original_satisfy_constraints, align 1
  %34 = load i1, i1* @constraint, align 1
  %35 = load i1, i1* %is_original_satisfy_constraints, align 1
  %36 = and i1 %35, %34
  store i1 %36, i1* %is_original_satisfy_constraints, align 1
  %outputNotEqual = icmp ne i128 %"cloned_result.gep.IsZero|out.output", %"original_result.gep.IsZero|out.output"
  %originalConstraintValue = load i1, i1* %is_original_satisfy_constraints, align 1
  %clonedConstraintValue = load i1, i1* %is_cloned_satisfy_constraints, align 1
  %tmp_under_constrained_condition = and i1 %outputNotEqual, %originalConstraintValue
  %final_under_constrained_condition = and i1 %tmp_under_constrained_condition, %clonedConstraintValue
  br i1 %final_under_constrained_condition, label %under_constrained_error, label %no_under_constrained_continue

under_constrained_error:                          ; preds = %entry
  %37 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([61 x i8], [61 x i8]* @0, i32 0, i32 0))
  call void @exit(i32 1)
  unreachable

no_under_constrained_continue:                    ; preds = %entry
  ret i32 0
}

attributes #0 = { nofree nosync nounwind readnone speculatable willreturn }
