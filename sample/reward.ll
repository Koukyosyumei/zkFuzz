; ModuleID = 'reward.circom'
source_filename = "../../benchmark/sample/reward.circom"

%struct_template_Reward = type { i128, i128 }

@constraint = external global i1

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

define %struct_template_Reward* @fn_template_build_Reward() {
entry:
  %malloccall = tail call i8* @malloc(i32 ptrtoint (%struct_template_Reward* getelementptr (%struct_template_Reward, %struct_template_Reward* null, i32 1) to i32))
  %struct_template_Reward = bitcast i8* %malloccall to %struct_template_Reward*
  ret %struct_template_Reward* %struct_template_Reward
}

declare noalias i8* @malloc(i32)

define void @fn_template_init_Reward(%struct_template_Reward* %0) {
entry:
  %initial.out.output = alloca i128, align 8
  %initial.gwei.var = alloca i128, align 8
  %initial.inp.input = alloca i128, align 8
  %"gep.Reward|inp.input" = getelementptr inbounds %struct_template_Reward, %struct_template_Reward* %0, i32 0, i32 0
  %read.inp.input = load i128, i128* %"gep.Reward|inp.input", align 4
  store i128 %read.inp.input, i128* %initial.inp.input, align 4
  br label %body

body:                                             ; preds = %entry
  %pow = call i128 @fn_intrinsic_utils_powi(i128 10, i128 6)
  store i128 %pow, i128* %initial.gwei.var, align 4
  %read.inp.input1 = load i128, i128* %initial.inp.input, align 4
  %read.gwei.var = load i128, i128* %initial.gwei.var, align 4
  %sdiv = sdiv i128 %read.inp.input1, %read.gwei.var
  store i128 %sdiv, i128* %initial.out.output, align 4
  %read.out.output = load i128, i128* %initial.out.output, align 4
  %read.gwei.var2 = load i128, i128* %initial.gwei.var, align 4
  %mul = mul i128 %read.out.output, %read.gwei.var2
  %read.inp.input3 = load i128, i128* %initial.inp.input, align 4
  call void @fn_intrinsic_utils_constraint(i128 %mul, i128 %read.inp.input3, i1* @constraint)
  br label %exit

exit:                                             ; preds = %body
  %read.out.output4 = load i128, i128* %initial.out.output, align 4
  %"gep.Reward|out.output" = getelementptr inbounds %struct_template_Reward, %struct_template_Reward* %0, i32 0, i32 1
  store i128 %read.out.output4, i128* %"gep.Reward|out.output", align 4
  ret void
}

attributes #0 = { nofree nosync nounwind readnone speculatable willreturn }
