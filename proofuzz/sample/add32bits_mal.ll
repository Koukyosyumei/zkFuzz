; ModuleID = 'add32bits_mal.circom'
source_filename = "../../benchmark/sample/add32bits_mal.circom"

%struct_template_MaliciousAdd32Bits = type { i128, i128, i128, i128 }

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

define %struct_template_MaliciousAdd32Bits* @fn_template_build_MaliciousAdd32Bits() {
entry:
  %malloccall = tail call i8* @malloc(i32 ptrtoint (%struct_template_MaliciousAdd32Bits* getelementptr (%struct_template_MaliciousAdd32Bits, %struct_template_MaliciousAdd32Bits* null, i32 1) to i32))
  %struct_template_MaliciousAdd32Bits = bitcast i8* %malloccall to %struct_template_MaliciousAdd32Bits*
  ret %struct_template_MaliciousAdd32Bits* %struct_template_MaliciousAdd32Bits
}

declare noalias i8* @malloc(i32)

define void @fn_template_init_MaliciousAdd32Bits(%struct_template_MaliciousAdd32Bits* %0) {
entry:
  %initial.tmp.inter = alloca i128, align 8
  %initial.b.input = alloca i128, align 8
  %"gep.MaliciousAdd32Bits|b.input" = getelementptr inbounds %struct_template_MaliciousAdd32Bits, %struct_template_MaliciousAdd32Bits* %0, i32 0, i32 1
  %read.b.input = load i128, i128* %"gep.MaliciousAdd32Bits|b.input", align 4
  store i128 %read.b.input, i128* %initial.b.input, align 4
  %initial.out.output = alloca i128, align 8
  %initial.a.input = alloca i128, align 8
  %"gep.MaliciousAdd32Bits|a.input" = getelementptr inbounds %struct_template_MaliciousAdd32Bits, %struct_template_MaliciousAdd32Bits* %0, i32 0, i32 0
  %read.a.input = load i128, i128* %"gep.MaliciousAdd32Bits|a.input", align 4
  store i128 %read.a.input, i128* %initial.a.input, align 4
  br label %body

body:                                             ; preds = %entry
  store i128 0, i128* %initial.tmp.inter, align 4
  %read.tmp.inter = load i128, i128* %initial.tmp.inter, align 4
  %read.tmp.inter1 = load i128, i128* %initial.tmp.inter, align 4
  %sub = sub i128 %read.tmp.inter1, 1
  %mul = mul i128 %read.tmp.inter, %sub
  call void @fn_intrinsic_utils_constraint(i128 %mul, i128 0, i1* @constraint)
  %read.a.input2 = load i128, i128* %initial.a.input, align 4
  %read.b.input3 = load i128, i128* %initial.b.input, align 4
  %add = add i128 %read.a.input2, %read.b.input3
  %read.tmp.inter4 = load i128, i128* %initial.tmp.inter, align 4
  %mul5 = mul i128 %read.tmp.inter4, 4294967296
  %sub6 = sub i128 %add, %mul5
  %read.out.output = load i128, i128* %initial.out.output, align 4
  call void @fn_intrinsic_utils_constraint(i128 %read.out.output, i128 %sub6, i1* @constraint.1)
  store i128 %sub6, i128* %initial.out.output, align 4
  br label %exit

exit:                                             ; preds = %body
  %read.tmp.inter7 = load i128, i128* %initial.tmp.inter, align 4
  %"gep.MaliciousAdd32Bits|tmp.inter" = getelementptr inbounds %struct_template_MaliciousAdd32Bits, %struct_template_MaliciousAdd32Bits* %0, i32 0, i32 2
  store i128 %read.tmp.inter7, i128* %"gep.MaliciousAdd32Bits|tmp.inter", align 4
  %read.out.output8 = load i128, i128* %initial.out.output, align 4
  %"gep.MaliciousAdd32Bits|out.output" = getelementptr inbounds %struct_template_MaliciousAdd32Bits, %struct_template_MaliciousAdd32Bits* %0, i32 0, i32 3
  store i128 %read.out.output8, i128* %"gep.MaliciousAdd32Bits|out.output", align 4
  ret void
}

attributes #0 = { nofree nosync nounwind readnone speculatable willreturn }
