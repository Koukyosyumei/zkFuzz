; ModuleID = './benchmark/sample/add32bits.ll'
source_filename = "./benchmark/sample/add32bits.circom"

%struct_template_Add32Bits = type { i128, i128, i128, i128 }

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

define %struct_template_Add32Bits* @fn_template_build_Add32Bits() {
entry:
  %malloccall = tail call i8* @malloc(i32 ptrtoint (%struct_template_Add32Bits* getelementptr (%struct_template_Add32Bits, %struct_template_Add32Bits* null, i32 1) to i32))
  %struct_template_Add32Bits = bitcast i8* %malloccall to %struct_template_Add32Bits*
  ret %struct_template_Add32Bits* %struct_template_Add32Bits
}

declare noalias i8* @malloc(i32)

define void @fn_template_init_Add32Bits(%struct_template_Add32Bits* %0) {
entry:
  %initial.out.output = alloca i128, align 8
  %initial.a.input = alloca i128, align 8
  %"gep.Add32Bits|a.input" = getelementptr inbounds %struct_template_Add32Bits, %struct_template_Add32Bits* %0, i32 0, i32 0
  %read.a.input = load i128, i128* %"gep.Add32Bits|a.input", align 4
  store i128 %read.a.input, i128* %initial.a.input, align 4
  %initial.b.input = alloca i128, align 8
  %"gep.Add32Bits|b.input" = getelementptr inbounds %struct_template_Add32Bits, %struct_template_Add32Bits* %0, i32 0, i32 1
  %read.b.input = load i128, i128* %"gep.Add32Bits|b.input", align 4
  store i128 %read.b.input, i128* %initial.b.input, align 4
  %initial.tmp.inter = alloca i128, align 8
  br label %body

body:                                             ; preds = %entry
  %read.a.input1 = load i128, i128* %initial.a.input, align 4
  %read.b.input2 = load i128, i128* %initial.b.input, align 4
  %mod_add = call i128 @mod_add(i128 %read.a.input1, i128 %read.b.input2, i128 9938766679346745377)
  %mod_add3 = call i128 @mod_add(i128 4294967295, i128 1, i128 9938766679346745377)
  %sge = icmp sge i128 %mod_add, %mod_add3
  %utils_switch = call i128 @fn_intrinsic_utils_switch(i1 %sge, i128 1, i128 0)
  store i128 %utils_switch, i128* %initial.tmp.inter, align 4
  %read.tmp.inter = load i128, i128* %initial.tmp.inter, align 4
  %read.tmp.inter4 = load i128, i128* %initial.tmp.inter, align 4
  %mod_sub = call i128 @mod_sub(i128 %read.tmp.inter4, i128 1, i128 9938766679346745377)
  %mod_mul = call i128 @mod_mul(i128 %read.tmp.inter, i128 %mod_sub, i128 9938766679346745377)
  call void @fn_intrinsic_utils_constraint(i128 %mod_mul, i128 0, i1* @constraint)
  %read.a.input5 = load i128, i128* %initial.a.input, align 4
  %read.b.input6 = load i128, i128* %initial.b.input, align 4
  %mod_add7 = call i128 @mod_add(i128 %read.a.input5, i128 %read.b.input6, i128 9938766679346745377)
  %read.tmp.inter8 = load i128, i128* %initial.tmp.inter, align 4
  %mod_add9 = call i128 @mod_add(i128 4294967295, i128 1, i128 9938766679346745377)
  %mod_mul10 = call i128 @mod_mul(i128 %read.tmp.inter8, i128 %mod_add9, i128 9938766679346745377)
  %mod_sub11 = call i128 @mod_sub(i128 %mod_add7, i128 %mod_mul10, i128 9938766679346745377)
  %read.out.output = load i128, i128* %initial.out.output, align 4
  call void @fn_intrinsic_utils_constraint(i128 %read.out.output, i128 %mod_sub11, i1* @constraint.1)
  store i128 %mod_sub11, i128* %initial.out.output, align 4
  br label %exit

exit:                                             ; preds = %body
  %read.tmp.inter12 = load i128, i128* %initial.tmp.inter, align 4
  %"gep.Add32Bits|tmp.inter" = getelementptr inbounds %struct_template_Add32Bits, %struct_template_Add32Bits* %0, i32 0, i32 2
  store i128 %read.tmp.inter12, i128* %"gep.Add32Bits|tmp.inter", align 4
  %read.out.output13 = load i128, i128* %initial.out.output, align 4
  %"gep.Add32Bits|out.output" = getelementptr inbounds %struct_template_Add32Bits, %struct_template_Add32Bits* %0, i32 0, i32 3
  store i128 %read.out.output13, i128* %"gep.Add32Bits|out.output", align 4
  ret void
}

declare i32 @printf(i8*, ...)

declare i32 @scanf(i8*, ...)

define i32 @main() {
entry:
  %instance = call %struct_template_Add32Bits* @fn_template_build_Add32Bits()
  %"gep.Add32Bits|b.input" = getelementptr %struct_template_Add32Bits, %struct_template_Add32Bits* %instance, i32 0, i32 1
  %0 = call i32 (i8*, ...) @scanf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.scanf, i32 0, i32 0), i128* %"gep.Add32Bits|b.input")
  %"gep.Add32Bits|a.input" = getelementptr %struct_template_Add32Bits, %struct_template_Add32Bits* %instance, i32 0, i32 0
  %1 = call i32 (i8*, ...) @scanf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.scanf, i32 0, i32 0), i128* %"gep.Add32Bits|a.input")
  call void @fn_template_init_Add32Bits(%struct_template_Add32Bits* %instance)
  %"gep.Add32Bits|out.output" = getelementptr %struct_template_Add32Bits, %struct_template_Add32Bits* %instance, i32 0, i32 3
  %"val.gep.Add32Bits|out.output" = load i128, i128* %"gep.Add32Bits|out.output", align 4
  %2 = trunc i128 %"val.gep.Add32Bits|out.output" to i64
  %3 = lshr i128 %"val.gep.Add32Bits|out.output", 64
  %4 = trunc i128 %3 to i64
  %5 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.printf, i32 0, i32 0), i64 %4)
  %6 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.printf, i32 0, i32 0), i64 %2)
  ret i32 0
}

attributes #0 = { nofree nosync nounwind readnone speculatable willreturn }
