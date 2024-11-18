; ModuleID = 'iszero_wo_constraints.circom'
source_filename = "../../benchmark/sample/iszero_wo_constraints.circom"

%struct_template_IsZero = type { i128, i128, i128, i128 }

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

define %struct_template_IsZero* @fn_template_build_IsZero() {
entry:
  %malloccall = tail call i8* @malloc(i32 ptrtoint (%struct_template_IsZero* getelementptr (%struct_template_IsZero, %struct_template_IsZero* null, i32 1) to i32))
  %struct_template_IsZero = bitcast i8* %malloccall to %struct_template_IsZero*
  ret %struct_template_IsZero* %struct_template_IsZero
}

declare noalias i8* @malloc(i32)

define void @fn_template_init_IsZero(%struct_template_IsZero* %0) {
entry:
  %initial.inv.inter = alloca i128, align 8
  %initial.out.output = alloca i128, align 8
  %initial.in.input = alloca i128, align 8
  %"gep.IsZero|in.input" = getelementptr inbounds %struct_template_IsZero, %struct_template_IsZero* %0, i32 0, i32 0
  %read.in.input = load i128, i128* %"gep.IsZero|in.input", align 4
  store i128 %read.in.input, i128* %initial.in.input, align 4
  %initial.tmp.inter = alloca i128, align 8
  br label %body

body:                                             ; preds = %entry
  %read.in.input1 = load i128, i128* %initial.in.input, align 4
  %ne = icmp ne i128 %read.in.input1, 0
  %read.in.input2 = load i128, i128* %initial.in.input, align 4
  %sdiv = sdiv i128 1, %read.in.input2
  %utils_switch = call i128 @fn_intrinsic_utils_switch(i1 %ne, i128 %sdiv, i128 0)
  store i128 %utils_switch, i128* %initial.inv.inter, align 4
  store i128 1, i128* %initial.tmp.inter, align 4
  %read.tmp.inter = load i128, i128* %initial.tmp.inter, align 4
  %neg = sub i128 0, %read.tmp.inter
  %add = add i128 %neg, 1
  store i128 %add, i128* %initial.out.output, align 4
  br label %exit

exit:                                             ; preds = %body
  %read.inv.inter = load i128, i128* %initial.inv.inter, align 4
  %"gep.IsZero|inv.inter" = getelementptr inbounds %struct_template_IsZero, %struct_template_IsZero* %0, i32 0, i32 1
  store i128 %read.inv.inter, i128* %"gep.IsZero|inv.inter", align 4
  %read.tmp.inter3 = load i128, i128* %initial.tmp.inter, align 4
  %"gep.IsZero|tmp.inter" = getelementptr inbounds %struct_template_IsZero, %struct_template_IsZero* %0, i32 0, i32 2
  store i128 %read.tmp.inter3, i128* %"gep.IsZero|tmp.inter", align 4
  %read.out.output = load i128, i128* %initial.out.output, align 4
  %"gep.IsZero|out.output" = getelementptr inbounds %struct_template_IsZero, %struct_template_IsZero* %0, i32 0, i32 3
  store i128 %read.out.output, i128* %"gep.IsZero|out.output", align 4
  ret void
}

attributes #0 = { nofree nosync nounwind readnone speculatable willreturn }
