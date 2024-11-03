; ModuleID = 'arrayxor.circom'
source_filename = "../../benchmark/sample/arrayxor.circom"

%struct_template_ArrayXOR = type { [256 x i128]*, [256 x i128]*, [256 x i128]* }

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

define %struct_template_ArrayXOR* @fn_template_build_ArrayXOR() {
entry:
  %malloccall = tail call i8* @malloc(i32 ptrtoint (%struct_template_ArrayXOR* getelementptr (%struct_template_ArrayXOR, %struct_template_ArrayXOR* null, i32 1) to i32))
  %struct_template_ArrayXOR = bitcast i8* %malloccall to %struct_template_ArrayXOR*
  ret %struct_template_ArrayXOR* %struct_template_ArrayXOR
}

declare noalias i8* @malloc(i32)

define void @fn_template_init_ArrayXOR(%struct_template_ArrayXOR* %0) {
entry:
  %initial.i.var = alloca i128, align 8
  %initial.out.output = alloca [256 x i128]*, align 8
  %malloccall = tail call i8* @malloc(i32 ptrtoint ([256 x i128]* getelementptr ([256 x i128], [256 x i128]* null, i32 1) to i32))
  %ArrayXORinlinearray = bitcast i8* %malloccall to [256 x i128]*
  store [256 x i128]* %ArrayXORinlinearray, [256 x i128]** %initial.out.output, align 8
  %initial.b.input = alloca [256 x i128]*, align 8
  %malloccall1 = tail call i8* @malloc(i32 ptrtoint ([256 x i128]* getelementptr ([256 x i128], [256 x i128]* null, i32 1) to i32))
  %ArrayXORinlinearray2 = bitcast i8* %malloccall1 to [256 x i128]*
  store [256 x i128]* %ArrayXORinlinearray2, [256 x i128]** %initial.b.input, align 8
  %"gep.ArrayXOR|b.input" = getelementptr inbounds %struct_template_ArrayXOR, %struct_template_ArrayXOR* %0, i32 0, i32 1
  %read.b.input = load [256 x i128]*, [256 x i128]** %"gep.ArrayXOR|b.input", align 8
  store [256 x i128]* %read.b.input, [256 x i128]** %initial.b.input, align 8
  %initial.a.input = alloca [256 x i128]*, align 8
  %malloccall3 = tail call i8* @malloc(i32 ptrtoint ([256 x i128]* getelementptr ([256 x i128], [256 x i128]* null, i32 1) to i32))
  %ArrayXORinlinearray4 = bitcast i8* %malloccall3 to [256 x i128]*
  store [256 x i128]* %ArrayXORinlinearray4, [256 x i128]** %initial.a.input, align 8
  %"gep.ArrayXOR|a.input" = getelementptr inbounds %struct_template_ArrayXOR, %struct_template_ArrayXOR* %0, i32 0, i32 0
  %read.a.input = load [256 x i128]*, [256 x i128]** %"gep.ArrayXOR|a.input", align 8
  store [256 x i128]* %read.a.input, [256 x i128]** %initial.a.input, align 8
  br label %body

body:                                             ; preds = %entry
  store i128 0, i128* %initial.i.var, align 4
  br label %loop.header

loop.header:                                      ; preds = %loop.body, %body
  %read.i.var = load i128, i128* %initial.i.var, align 4
  %slt = icmp slt i128 %read.i.var, 2
  br i1 %slt, label %loop.body, label %loop.exit

loop.body:                                        ; preds = %loop.header
  %var_getter = load [256 x i128]*, [256 x i128]** %initial.a.input, align 8
  %read.i.var5 = load i128, i128* %initial.i.var, align 4
  %array_getter = getelementptr inbounds [256 x i128], [256 x i128]* %var_getter, i128 0, i128 %read.i.var5
  %read.a.input6 = load i128, i128* %array_getter, align 4
  %var_getter7 = load [256 x i128]*, [256 x i128]** %initial.b.input, align 8
  %read.i.var8 = load i128, i128* %initial.i.var, align 4
  %array_getter9 = getelementptr inbounds [256 x i128], [256 x i128]* %var_getter7, i128 0, i128 %read.i.var8
  %read.b.input10 = load i128, i128* %array_getter9, align 4
  %xor = xor i128 %read.a.input6, %read.b.input10
  %var_getter11 = load [256 x i128]*, [256 x i128]** %initial.out.output, align 8
  %read.i.var12 = load i128, i128* %initial.i.var, align 4
  %write.out.output = getelementptr inbounds [256 x i128], [256 x i128]* %var_getter11, i128 0, i128 %read.i.var12
  store i128 %xor, i128* %write.out.output, align 4
  %read.i.var13 = load i128, i128* %initial.i.var, align 4
  %add = add i128 %read.i.var13, 1
  store i128 %add, i128* %initial.i.var, align 4
  br label %loop.header

loop.exit:                                        ; preds = %loop.header
  br label %exit

exit:                                             ; preds = %loop.exit
  %ptr_cast = bitcast [256 x i128]** %initial.out.output to i128*
  call void (i128*, ...) @fn_intrinsic_utils_arraydim(i128* %ptr_cast, i128 2)
  %ptr_cast14 = bitcast [256 x i128]** %initial.a.input to i128*
  call void (i128*, ...) @fn_intrinsic_utils_arraydim(i128* %ptr_cast14, i128 2)
  %ptr_cast15 = bitcast [256 x i128]** %initial.b.input to i128*
  call void (i128*, ...) @fn_intrinsic_utils_arraydim(i128* %ptr_cast15, i128 2)
  %read.out.output = load [256 x i128]*, [256 x i128]** %initial.out.output, align 8
  %"gep.ArrayXOR|out.output" = getelementptr inbounds %struct_template_ArrayXOR, %struct_template_ArrayXOR* %0, i32 0, i32 2
  store [256 x i128]* %read.out.output, [256 x i128]** %"gep.ArrayXOR|out.output", align 8
  ret void
}

attributes #0 = { nofree nosync nounwind readnone speculatable willreturn }
