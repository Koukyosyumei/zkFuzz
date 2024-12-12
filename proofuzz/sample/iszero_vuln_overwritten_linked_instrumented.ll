; ModuleID = './benchmark/sample/iszero_vuln_overwritten_linked.ll'
source_filename = "llvm-link"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

%struct_template_IsZero = type { i128, i128, i128 }

$fn_intrinsic_utils_constraint = comdat nodeduplicate

$fn_intrinsic_utils_constraint_array = comdat nodeduplicate

$fn_intrinsic_utils_switch = comdat nodeduplicate

$fn_intrinsic_utils_powi = comdat nodeduplicate

$fn_intrinsic_utils_init = comdat nodeduplicate

$fn_intrinsic_utils_assert = comdat nodeduplicate

$fn_intrinsic_utils_arraydim = comdat nodeduplicate

$fn_template_build_IsZero = comdat nodeduplicate

$fn_template_init_IsZero = comdat nodeduplicate

$cloned_fn_template_init_IsZero = comdat nodeduplicate

$main = comdat nodeduplicate

$mod_add = comdat nodeduplicate

$mod_sub = comdat nodeduplicate

$mod_mul = comdat nodeduplicate

$mod_inverse = comdat nodeduplicate

$mod_div = comdat nodeduplicate

$sancov.module_ctor_trace_pc_guard = comdat any

@constraint = internal unnamed_addr global i1 false
@constraint.1 = internal unnamed_addr global i1 false
@.str.map.d = private constant [4 x i8] c"%d\0A\00"
@.str.map.ld = private constant [5 x i8] c"%ld\0A\00"
@.str.map.lld = private constant [5 x i8] c"%lld\00"
@str = private unnamed_addr constant [60 x i8] c"Error: Under-Constraint-Condition Met. Terminating program.\00", align 1
@__afl_area_ptr = external global i8*
@__sancov_gen_ = private global [2 x i32] zeroinitializer, section "__sancov_guards", comdat($fn_intrinsic_utils_constraint), align 4
@__sancov_gen_.1 = private global [1 x i32] zeroinitializer, section "__sancov_guards", comdat($fn_intrinsic_utils_constraint_array), align 4
@__sancov_gen_.2 = private global [3 x i32] zeroinitializer, section "__sancov_guards", comdat($fn_intrinsic_utils_switch), align 4
@__sancov_gen_.3 = private global [1 x i32] zeroinitializer, section "__sancov_guards", comdat($fn_intrinsic_utils_powi), align 4
@__sancov_gen_.4 = private global [1 x i32] zeroinitializer, section "__sancov_guards", comdat($fn_intrinsic_utils_init), align 4
@__sancov_gen_.5 = private global [1 x i32] zeroinitializer, section "__sancov_guards", comdat($fn_intrinsic_utils_assert), align 4
@__sancov_gen_.6 = private global [1 x i32] zeroinitializer, section "__sancov_guards", comdat($fn_intrinsic_utils_arraydim), align 4
@__sancov_gen_.7 = private global [1 x i32] zeroinitializer, section "__sancov_guards", comdat($fn_template_build_IsZero), align 4
@__sancov_gen_.8 = private global [22 x i32] zeroinitializer, section "__sancov_guards", comdat($fn_template_init_IsZero), align 4
@__sancov_gen_.9 = private global [18 x i32] zeroinitializer, section "__sancov_guards", comdat($cloned_fn_template_init_IsZero), align 4
@__sancov_gen_.10 = private global [39 x i32] zeroinitializer, section "__sancov_guards", comdat($main), align 4
@__sancov_gen_.11 = private global [3 x i32] zeroinitializer, section "__sancov_guards", comdat($mod_add), align 4
@__sancov_gen_.12 = private global [3 x i32] zeroinitializer, section "__sancov_guards", comdat($mod_sub), align 4
@__sancov_gen_.13 = private global [3 x i32] zeroinitializer, section "__sancov_guards", comdat($mod_mul), align 4
@__sancov_gen_.14 = private global [10 x i32] zeroinitializer, section "__sancov_guards", comdat($mod_inverse), align 4
@__sancov_gen_.15 = private global [13 x i32] zeroinitializer, section "__sancov_guards", comdat($mod_div), align 4
@__start___sancov_guards = extern_weak hidden global i32*
@__stop___sancov_guards = extern_weak hidden global i32*
@llvm.global_ctors = appending global [1 x { i32, void ()*, i8* }] [{ i32, void ()*, i8* } { i32 2, void ()* @sancov.module_ctor_trace_pc_guard, i8* bitcast (void ()* @sancov.module_ctor_trace_pc_guard to i8*) }]
@llvm.used = appending global [1 x i8*] [i8* bitcast (void ()* @sancov.module_ctor_trace_pc_guard to i8*)], section "llvm.metadata"
@llvm.compiler.used = appending global [16 x i8*] [i8* bitcast ([2 x i32]* @__sancov_gen_ to i8*), i8* bitcast ([1 x i32]* @__sancov_gen_.1 to i8*), i8* bitcast ([3 x i32]* @__sancov_gen_.2 to i8*), i8* bitcast ([1 x i32]* @__sancov_gen_.3 to i8*), i8* bitcast ([1 x i32]* @__sancov_gen_.4 to i8*), i8* bitcast ([1 x i32]* @__sancov_gen_.5 to i8*), i8* bitcast ([1 x i32]* @__sancov_gen_.6 to i8*), i8* bitcast ([1 x i32]* @__sancov_gen_.7 to i8*), i8* bitcast ([22 x i32]* @__sancov_gen_.8 to i8*), i8* bitcast ([18 x i32]* @__sancov_gen_.9 to i8*), i8* bitcast ([39 x i32]* @__sancov_gen_.10 to i8*), i8* bitcast ([3 x i32]* @__sancov_gen_.11 to i8*), i8* bitcast ([3 x i32]* @__sancov_gen_.12 to i8*), i8* bitcast ([3 x i32]* @__sancov_gen_.13 to i8*), i8* bitcast ([10 x i32]* @__sancov_gen_.14 to i8*), i8* bitcast ([13 x i32]* @__sancov_gen_.15 to i8*)], section "llvm.metadata"

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn writeonly
define void @fn_intrinsic_utils_constraint(i128 %0, i128 %1, i1* nocapture writeonly %2) local_unnamed_addr #0 comdat {
entry:
  %3 = load i32, i32* getelementptr inbounds ([2 x i32], [2 x i32]* @__sancov_gen_, i32 0, i32 0), align 4, !nosanitize !0
  %4 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %5 = getelementptr i8, i8* %4, i32 %3
  %6 = load i8, i8* %5, align 1, !nosanitize !0
  %7 = add i8 %6, 1
  %8 = icmp eq i8 %7, 0
  %9 = zext i1 %8 to i8
  %10 = add i8 %7, %9
  store i8 %10, i8* %5, align 1, !nosanitize !0
  %constraint = icmp eq i128 %0, %1
  store i1 %constraint, i1* %2, align 1
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind readnone willreturn
define void @fn_intrinsic_utils_constraint_array([256 x i128]* nocapture readnone %0, [256 x i128]* nocapture readnone %1, i1* nocapture readnone %2) local_unnamed_addr #1 comdat {
entry:
  %3 = load i32, i32* getelementptr inbounds ([1 x i32], [1 x i32]* @__sancov_gen_.1, i32 0, i32 0), align 4, !nosanitize !0
  %4 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %5 = getelementptr i8, i8* %4, i32 %3
  %6 = load i8, i8* %5, align 1, !nosanitize !0
  %7 = add i8 %6, 1
  %8 = icmp eq i8 %7, 0
  %9 = zext i1 %8 to i8
  %10 = add i8 %7, %9
  store i8 %10, i8* %5, align 1, !nosanitize !0
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind readnone willreturn
define i128 @fn_intrinsic_utils_switch(i1 %0, i128 %1, i128 %2) local_unnamed_addr #1 comdat {
entry:
  %3 = load i32, i32* getelementptr inbounds ([3 x i32], [3 x i32]* @__sancov_gen_.2, i32 0, i32 0), align 4, !nosanitize !0
  %4 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %5 = getelementptr i8, i8* %4, i32 %3
  %6 = load i8, i8* %5, align 1, !nosanitize !0
  %7 = add i8 %6, 1
  %8 = icmp eq i8 %7, 0
  %9 = zext i1 %8 to i8
  %10 = add i8 %7, %9
  store i8 %10, i8* %5, align 1, !nosanitize !0
  %. = select i1 %0, i128 %1, i128 %2
  %11 = select i1 %0, i32* inttoptr (i64 add (i64 ptrtoint ([3 x i32]* @__sancov_gen_.2 to i64), i64 4) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([3 x i32]* @__sancov_gen_.2 to i64), i64 8) to i32*)
  %12 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %13 = load i32, i32* %11, align 4, !nosanitize !0
  %14 = getelementptr i8, i8* %12, i32 %13
  %15 = load i8, i8* %14, align 1, !nosanitize !0
  %16 = add i8 %15, 1
  %17 = icmp eq i8 %16, 0
  %18 = zext i1 %17 to i8
  %19 = add i8 %16, %18
  store i8 %19, i8* %14, align 1, !nosanitize !0
  ret i128 %.
}

; Function Attrs: mustprogress nofree nosync nounwind readnone willreturn
define i128 @fn_intrinsic_utils_powi(i128 %0, i128 %1) local_unnamed_addr #2 comdat {
entry:
  %2 = load i32, i32* getelementptr inbounds ([1 x i32], [1 x i32]* @__sancov_gen_.3, i32 0, i32 0), align 4, !nosanitize !0
  %3 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %4 = getelementptr i8, i8* %3, i32 %2
  %5 = load i8, i8* %4, align 1, !nosanitize !0
  %6 = add i8 %5, 1
  %7 = icmp eq i8 %6, 0
  %8 = zext i1 %7 to i8
  %9 = add i8 %6, %8
  store i8 %9, i8* %4, align 1, !nosanitize !0
  %utils_powi.base = uitofp i128 %0 to fp128
  %utils_powi.power = trunc i128 %1 to i32
  %utils_powi.cal = tail call fp128 @llvm.powi.f128.i32(fp128 %utils_powi.base, i32 %utils_powi.power)
  %utils_powi.ret = fptoui fp128 %utils_powi.cal to i128
  ret i128 %utils_powi.ret
}

; Function Attrs: mustprogress nofree nosync nounwind readnone speculatable willreturn
declare fp128 @llvm.powi.f128.i32(fp128, i32) #3

; Function Attrs: mustprogress nofree norecurse nosync nounwind readnone willreturn
define i128 @fn_intrinsic_utils_init() local_unnamed_addr #1 comdat {
entry:
  %0 = load i32, i32* getelementptr inbounds ([1 x i32], [1 x i32]* @__sancov_gen_.4, i32 0, i32 0), align 4, !nosanitize !0
  %1 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %2 = getelementptr i8, i8* %1, i32 %0
  %3 = load i8, i8* %2, align 1, !nosanitize !0
  %4 = add i8 %3, 1
  %5 = icmp eq i8 %4, 0
  %6 = zext i1 %5 to i8
  %7 = add i8 %4, %6
  store i8 %7, i8* %2, align 1, !nosanitize !0
  ret i128 0
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind readnone willreturn
define void @fn_intrinsic_utils_assert(i1 %0) local_unnamed_addr #1 comdat {
entry:
  %1 = load i32, i32* getelementptr inbounds ([1 x i32], [1 x i32]* @__sancov_gen_.5, i32 0, i32 0), align 4, !nosanitize !0
  %2 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %3 = getelementptr i8, i8* %2, i32 %1
  %4 = load i8, i8* %3, align 1, !nosanitize !0
  %5 = add i8 %4, 1
  %6 = icmp eq i8 %5, 0
  %7 = zext i1 %6 to i8
  %8 = add i8 %5, %7
  store i8 %8, i8* %3, align 1, !nosanitize !0
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind readnone willreturn
define void @fn_intrinsic_utils_arraydim(i128* nocapture readnone %0, ...) local_unnamed_addr #1 comdat {
entry:
  %1 = load i32, i32* getelementptr inbounds ([1 x i32], [1 x i32]* @__sancov_gen_.6, i32 0, i32 0), align 4, !nosanitize !0
  %2 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %3 = getelementptr i8, i8* %2, i32 %1
  %4 = load i8, i8* %3, align 1, !nosanitize !0
  %5 = add i8 %4, 1
  %6 = icmp eq i8 %5, 0
  %7 = zext i1 %6 to i8
  %8 = add i8 %5, %7
  store i8 %8, i8* %3, align 1, !nosanitize !0
  ret void
}

; Function Attrs: mustprogress nofree nounwind willreturn
define noalias %struct_template_IsZero* @fn_template_build_IsZero() local_unnamed_addr #4 comdat {
entry:
  %0 = load i32, i32* getelementptr inbounds ([1 x i32], [1 x i32]* @__sancov_gen_.7, i32 0, i32 0), align 4, !nosanitize !0
  %1 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %2 = getelementptr i8, i8* %1, i32 %0
  %3 = load i8, i8* %2, align 1, !nosanitize !0
  %4 = add i8 %3, 1
  %5 = icmp eq i8 %4, 0
  %6 = zext i1 %5 to i8
  %7 = add i8 %4, %6
  store i8 %7, i8* %2, align 1, !nosanitize !0
  %malloccall = tail call dereferenceable_or_null(48) i8* @malloc(i32 48)
  %struct_template_IsZero = bitcast i8* %malloccall to %struct_template_IsZero*
  ret %struct_template_IsZero* %struct_template_IsZero
}

; Function Attrs: inaccessiblememonly mustprogress nofree nounwind willreturn
declare noalias noundef i8* @malloc(i32 noundef) local_unnamed_addr #5

; Function Attrs: nounwind
define void @fn_template_init_IsZero(%struct_template_IsZero* nocapture %0) local_unnamed_addr #6 comdat {
entry:
  %1 = load i32, i32* getelementptr inbounds ([22 x i32], [22 x i32]* @__sancov_gen_.8, i32 0, i32 0), align 4, !nosanitize !0
  %2 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %3 = getelementptr i8, i8* %2, i32 %1
  %4 = load i8, i8* %3, align 1, !nosanitize !0
  %5 = add i8 %4, 1
  %6 = icmp eq i8 %5, 0
  %7 = zext i1 %6 to i8
  %8 = add i8 %5, %7
  store i8 %8, i8* %3, align 1, !nosanitize !0
  %"gep.IsZero|in.input" = getelementptr inbounds %struct_template_IsZero, %struct_template_IsZero* %0, i64 0, i32 0
  %read.in.input = load i128, i128* %"gep.IsZero|in.input", align 4
  %b_is_zero.i = icmp eq i128 %read.in.input, 0
  br i1 %b_is_zero.i, label %entry.mod_div.exit_crit_edge, label %compute_inverse.i

entry.mod_div.exit_crit_edge:                     ; preds = %entry
  %9 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([22 x i32]* @__sancov_gen_.8 to i64), i64 4) to i32*), align 4, !nosanitize !0
  %10 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %11 = getelementptr i8, i8* %10, i32 %9
  %12 = load i8, i8* %11, align 1, !nosanitize !0
  %13 = add i8 %12, 1
  %14 = icmp eq i8 %13, 0
  %15 = zext i1 %14 to i8
  %16 = add i8 %13, %15
  store i8 %16, i8* %11, align 1, !nosanitize !0
  br label %mod_div.exit

compute_inverse.i:                                ; preds = %entry
  %new_r_neg.i.i = icmp slt i128 %read.in.input, 0
  %new_r_add.i.i = select i1 %new_r_neg.i.i, i128 9938766679346745377, i128 0
  %17 = select i1 %new_r_neg.i.i, i32* inttoptr (i64 add (i64 ptrtoint ([22 x i32]* @__sancov_gen_.8 to i64), i64 24) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([22 x i32]* @__sancov_gen_.8 to i64), i64 28) to i32*)
  %18 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %19 = load i32, i32* %17, align 4, !nosanitize !0
  %20 = getelementptr i8, i8* %18, i32 %19
  %21 = load i8, i8* %20, align 1, !nosanitize !0
  %22 = add i8 %21, 1
  %23 = icmp eq i8 %22, 0
  %24 = zext i1 %23 to i8
  %25 = add i8 %22, %24
  store i8 %25, i8* %20, align 1, !nosanitize !0
  %new_r_abs.i.i = add i128 %new_r_add.i.i, %read.in.input
  %is_zero3.i.i = icmp eq i128 %new_r_abs.i.i, 0
  br i1 %is_zero3.i.i, label %compute_inverse.i.error.i.i_crit_edge, label %compute_inverse.i.continue.i.i_crit_edge

compute_inverse.i.continue.i.i_crit_edge:         ; preds = %compute_inverse.i
  br label %continue.i.i

compute_inverse.i.error.i.i_crit_edge:            ; preds = %compute_inverse.i
  %26 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([22 x i32]* @__sancov_gen_.8 to i64), i64 8) to i32*), align 4, !nosanitize !0
  %27 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %28 = getelementptr i8, i8* %27, i32 %26
  %29 = load i8, i8* %28, align 1, !nosanitize !0
  %30 = add i8 %29, 1
  %31 = icmp eq i8 %30, 0
  %32 = zext i1 %31 to i8
  %33 = add i8 %30, %32
  store i8 %33, i8* %28, align 1, !nosanitize !0
  br label %error.i.i

continue.i.i:                                     ; preds = %continue.i.i.continue.i.i_crit_edge, %compute_inverse.i.continue.i.i_crit_edge
  %new_r.07.i.i = phi i128 [ %new_r_updated.i.i, %continue.i.i.continue.i.i_crit_edge ], [ %new_r_abs.i.i, %compute_inverse.i.continue.i.i_crit_edge ]
  %r.06.i.i = phi i128 [ %new_r.07.i.i, %continue.i.i.continue.i.i_crit_edge ], [ 9938766679346745377, %compute_inverse.i.continue.i.i_crit_edge ]
  %new_t.05.i.i = phi i128 [ %new_t_updated.i.i, %continue.i.i.continue.i.i_crit_edge ], [ 1, %compute_inverse.i.continue.i.i_crit_edge ]
  %t.04.i.i = phi i128 [ %new_t.05.i.i, %continue.i.i.continue.i.i_crit_edge ], [ 0, %compute_inverse.i.continue.i.i_crit_edge ]
  %quotient.i.i = sdiv i128 %r.06.i.i, %new_r.07.i.i
  %temp_t.i.i = mul i128 %quotient.i.i, %new_t.05.i.i
  %new_t_updated.i.i = sub i128 %t.04.i.i, %temp_t.i.i
  %temp_r.i.i = mul i128 %quotient.i.i, %new_r.07.i.i
  %new_r_updated.i.i = sub i128 %r.06.i.i, %temp_r.i.i
  %is_zero.i.i = icmp eq i128 %new_r_updated.i.i, 0
  br i1 %is_zero.i.i, label %end.i.i, label %continue.i.i.continue.i.i_crit_edge

continue.i.i.continue.i.i_crit_edge:              ; preds = %continue.i.i
  %34 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([22 x i32]* @__sancov_gen_.8 to i64), i64 12) to i32*), align 4, !nosanitize !0
  %35 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %36 = getelementptr i8, i8* %35, i32 %34
  %37 = load i8, i8* %36, align 1, !nosanitize !0
  %38 = add i8 %37, 1
  %39 = icmp eq i8 %38, 0
  %40 = zext i1 %39 to i8
  %41 = add i8 %38, %40
  store i8 %41, i8* %36, align 1, !nosanitize !0
  br label %continue.i.i

end.i.i:                                          ; preds = %continue.i.i
  %inverse_exists.i.i = icmp eq i128 %new_r.07.i.i, 1
  br i1 %inverse_exists.i.i, label %mod_inverse.exit.i, label %end.i.i.error.i.i_crit_edge

end.i.i.error.i.i_crit_edge:                      ; preds = %end.i.i
  %42 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([22 x i32]* @__sancov_gen_.8 to i64), i64 16) to i32*), align 4, !nosanitize !0
  %43 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %44 = getelementptr i8, i8* %43, i32 %42
  %45 = load i8, i8* %44, align 1, !nosanitize !0
  %46 = add i8 %45, 1
  %47 = icmp eq i8 %46, 0
  %48 = zext i1 %47 to i8
  %49 = add i8 %46, %48
  store i8 %49, i8* %44, align 1, !nosanitize !0
  br label %error.i.i

error.i.i:                                        ; preds = %end.i.i.error.i.i_crit_edge, %compute_inverse.i.error.i.i_crit_edge
  tail call void @llvm.trap() #6
  unreachable

mod_inverse.exit.i:                               ; preds = %end.i.i
  %50 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([22 x i32]* @__sancov_gen_.8 to i64), i64 20) to i32*), align 4, !nosanitize !0
  %51 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %52 = getelementptr i8, i8* %51, i32 %50
  %53 = load i8, i8* %52, align 1, !nosanitize !0
  %54 = add i8 %53, 1
  %55 = icmp eq i8 %54, 0
  %56 = zext i1 %55 to i8
  %57 = add i8 %54, %56
  store i8 %57, i8* %52, align 1, !nosanitize !0
  %result_neg.i.i = icmp slt i128 %new_t.05.i.i, 0
  %result_add.i.i = select i1 %result_neg.i.i, i128 9938766679346745377, i128 0
  %58 = select i1 %result_neg.i.i, i32* inttoptr (i64 add (i64 ptrtoint ([22 x i32]* @__sancov_gen_.8 to i64), i64 32) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([22 x i32]* @__sancov_gen_.8 to i64), i64 36) to i32*)
  %59 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %60 = load i32, i32* %58, align 4, !nosanitize !0
  %61 = getelementptr i8, i8* %59, i32 %60
  %62 = load i8, i8* %61, align 1, !nosanitize !0
  %63 = add i8 %62, 1
  %64 = icmp eq i8 %63, 0
  %65 = zext i1 %64 to i8
  %66 = add i8 %63, %65
  store i8 %66, i8* %61, align 1, !nosanitize !0
  %result_pos.i.i = add i128 %result_add.i.i, %new_t.05.i.i
  %final_result.i.i = srem i128 %result_pos.i.i, 9938766679346745377
  %is_neg.i.i = icmp slt i128 %final_result.i.i, 0
  %pos_result.i.i = select i1 %is_neg.i.i, i128 9938766679346745377, i128 0
  %67 = select i1 %is_neg.i.i, i32* inttoptr (i64 add (i64 ptrtoint ([22 x i32]* @__sancov_gen_.8 to i64), i64 40) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([22 x i32]* @__sancov_gen_.8 to i64), i64 44) to i32*)
  %68 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %69 = load i32, i32* %67, align 4, !nosanitize !0
  %70 = getelementptr i8, i8* %68, i32 %69
  %71 = load i8, i8* %70, align 1, !nosanitize !0
  %72 = add i8 %71, 1
  %73 = icmp eq i8 %72, 0
  %74 = zext i1 %73 to i8
  %75 = add i8 %72, %74
  store i8 %75, i8* %70, align 1, !nosanitize !0
  %final_result.i1.i = add nsw i128 %pos_result.i.i, %final_result.i.i
  br label %mod_div.exit

mod_div.exit:                                     ; preds = %entry.mod_div.exit_crit_edge, %mod_inverse.exit.i
  %..i = phi i128 [ 0, %entry.mod_div.exit_crit_edge ], [ %final_result.i1.i, %mod_inverse.exit.i ]
  %sub.i = sub i128 0, %read.in.input
  %result.i = srem i128 %sub.i, 9938766679346745377
  %is_neg.i = icmp slt i128 %result.i, 0
  %pos_result.i = select i1 %is_neg.i, i128 9938766679346745377, i128 0
  %76 = select i1 %is_neg.i, i32* inttoptr (i64 add (i64 ptrtoint ([22 x i32]* @__sancov_gen_.8 to i64), i64 48) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([22 x i32]* @__sancov_gen_.8 to i64), i64 52) to i32*)
  %77 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %78 = load i32, i32* %76, align 4, !nosanitize !0
  %79 = getelementptr i8, i8* %77, i32 %78
  %80 = load i8, i8* %79, align 1, !nosanitize !0
  %81 = add i8 %80, 1
  %82 = icmp eq i8 %81, 0
  %83 = zext i1 %82 to i8
  %84 = add i8 %81, %83
  store i8 %84, i8* %79, align 1, !nosanitize !0
  %final_result.i = add nsw i128 %pos_result.i, %result.i
  %prod.i = mul i128 %..i, %final_result.i
  %result.i7 = srem i128 %prod.i, 9938766679346745377
  %is_neg.i8 = icmp slt i128 %result.i7, 0
  %pos_result.i9 = select i1 %is_neg.i8, i128 9938766679346745377, i128 0
  %85 = select i1 %is_neg.i8, i32* inttoptr (i64 add (i64 ptrtoint ([22 x i32]* @__sancov_gen_.8 to i64), i64 56) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([22 x i32]* @__sancov_gen_.8 to i64), i64 60) to i32*)
  %86 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %87 = load i32, i32* %85, align 4, !nosanitize !0
  %88 = getelementptr i8, i8* %86, i32 %87
  %89 = load i8, i8* %88, align 1, !nosanitize !0
  %90 = add i8 %89, 1
  %91 = icmp eq i8 %90, 0
  %92 = zext i1 %91 to i8
  %93 = add i8 %90, %92
  store i8 %93, i8* %88, align 1, !nosanitize !0
  %final_result.i10 = add nsw i128 %result.i7, 1
  %add.i = add nsw i128 %final_result.i10, %pos_result.i9
  %result.i11 = srem i128 %add.i, 9938766679346745377
  %is_neg.i12 = icmp slt i128 %result.i11, 0
  %pos_result.i13 = select i1 %is_neg.i12, i128 9938766679346745377, i128 0
  %94 = select i1 %is_neg.i12, i32* inttoptr (i64 add (i64 ptrtoint ([22 x i32]* @__sancov_gen_.8 to i64), i64 64) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([22 x i32]* @__sancov_gen_.8 to i64), i64 68) to i32*)
  %95 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %96 = load i32, i32* %94, align 4, !nosanitize !0
  %97 = getelementptr i8, i8* %95, i32 %96
  %98 = load i8, i8* %97, align 1, !nosanitize !0
  %99 = add i8 %98, 1
  %100 = icmp eq i8 %99, 0
  %101 = zext i1 %100 to i8
  %102 = add i8 %99, %101
  store i8 %102, i8* %97, align 1, !nosanitize !0
  %final_result.i14 = add nsw i128 %pos_result.i13, %result.i11
  store i1 true, i1* @constraint, align 1
  %sub.i15 = add nsw i128 %final_result.i14, -1
  %result.i16 = srem i128 %sub.i15, 9938766679346745377
  %is_neg.i17 = icmp slt i128 %result.i16, 0
  %pos_result.i18 = select i1 %is_neg.i17, i128 9938766679346745377, i128 0
  %103 = select i1 %is_neg.i17, i32* inttoptr (i64 add (i64 ptrtoint ([22 x i32]* @__sancov_gen_.8 to i64), i64 72) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([22 x i32]* @__sancov_gen_.8 to i64), i64 76) to i32*)
  %104 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %105 = load i32, i32* %103, align 4, !nosanitize !0
  %106 = getelementptr i8, i8* %104, i32 %105
  %107 = load i8, i8* %106, align 1, !nosanitize !0
  %108 = add i8 %107, 1
  %109 = icmp eq i8 %108, 0
  %110 = zext i1 %109 to i8
  %111 = add i8 %108, %110
  store i8 %111, i8* %106, align 1, !nosanitize !0
  %final_result.i19 = add nsw i128 %pos_result.i18, %result.i16
  %prod.i20 = mul i128 %final_result.i19, %final_result.i14
  %result.i21 = srem i128 %prod.i20, 9938766679346745377
  %is_neg.i22 = icmp slt i128 %result.i21, 0
  %pos_result.i23 = select i1 %is_neg.i22, i128 9938766679346745377, i128 0
  %112 = select i1 %is_neg.i22, i32* inttoptr (i64 add (i64 ptrtoint ([22 x i32]* @__sancov_gen_.8 to i64), i64 80) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([22 x i32]* @__sancov_gen_.8 to i64), i64 84) to i32*)
  %113 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %114 = load i32, i32* %112, align 4, !nosanitize !0
  %115 = getelementptr i8, i8* %113, i32 %114
  %116 = load i8, i8* %115, align 1, !nosanitize !0
  %117 = add i8 %116, 1
  %118 = icmp eq i8 %117, 0
  %119 = zext i1 %118 to i8
  %120 = add i8 %117, %119
  store i8 %120, i8* %115, align 1, !nosanitize !0
  %final_result.i24 = sub nsw i128 0, %result.i21
  %constraint.i = icmp eq i128 %pos_result.i23, %final_result.i24
  store i1 %constraint.i, i1* @constraint.1, align 1
  %"gep.IsZero|inv.inter" = getelementptr inbounds %struct_template_IsZero, %struct_template_IsZero* %0, i64 0, i32 1
  store i128 %..i, i128* %"gep.IsZero|inv.inter", align 4
  %"gep.IsZero|out.output" = getelementptr inbounds %struct_template_IsZero, %struct_template_IsZero* %0, i64 0, i32 2
  store i128 %final_result.i14, i128* %"gep.IsZero|out.output", align 4
  ret void
}

; Function Attrs: nounwind
define void @cloned_fn_template_init_IsZero(%struct_template_IsZero* nocapture %0) local_unnamed_addr #6 comdat {
entry:
  %1 = load i32, i32* getelementptr inbounds ([18 x i32], [18 x i32]* @__sancov_gen_.9, i32 0, i32 0), align 4, !nosanitize !0
  %2 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %3 = getelementptr i8, i8* %2, i32 %1
  %4 = load i8, i8* %3, align 1, !nosanitize !0
  %5 = add i8 %4, 1
  %6 = icmp eq i8 %5, 0
  %7 = zext i1 %6 to i8
  %8 = add i8 %5, %7
  store i8 %8, i8* %3, align 1, !nosanitize !0
  %"gep.IsZero|in.input" = getelementptr inbounds %struct_template_IsZero, %struct_template_IsZero* %0, i64 0, i32 0
  %read.in.input = load i128, i128* %"gep.IsZero|in.input", align 4
  %"free.gep.IsZero|inv.inter" = getelementptr %struct_template_IsZero, %struct_template_IsZero* %0, i64 0, i32 1
  %free.read.inv.inter = load i128, i128* %"free.gep.IsZero|inv.inter", align 4
  %b_is_zero.i = icmp eq i128 %read.in.input, 0
  br i1 %b_is_zero.i, label %entry.mod_div.exit_crit_edge, label %compute_inverse.i

entry.mod_div.exit_crit_edge:                     ; preds = %entry
  %9 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([18 x i32]* @__sancov_gen_.9 to i64), i64 4) to i32*), align 4, !nosanitize !0
  %10 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %11 = getelementptr i8, i8* %10, i32 %9
  %12 = load i8, i8* %11, align 1, !nosanitize !0
  %13 = add i8 %12, 1
  %14 = icmp eq i8 %13, 0
  %15 = zext i1 %14 to i8
  %16 = add i8 %13, %15
  store i8 %16, i8* %11, align 1, !nosanitize !0
  br label %mod_div.exit

compute_inverse.i:                                ; preds = %entry
  %new_r_neg.i.i = icmp slt i128 %read.in.input, 0
  %new_r_add.i.i = select i1 %new_r_neg.i.i, i128 9938766679346745377, i128 0
  %17 = select i1 %new_r_neg.i.i, i32* inttoptr (i64 add (i64 ptrtoint ([18 x i32]* @__sancov_gen_.9 to i64), i64 24) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([18 x i32]* @__sancov_gen_.9 to i64), i64 28) to i32*)
  %18 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %19 = load i32, i32* %17, align 4, !nosanitize !0
  %20 = getelementptr i8, i8* %18, i32 %19
  %21 = load i8, i8* %20, align 1, !nosanitize !0
  %22 = add i8 %21, 1
  %23 = icmp eq i8 %22, 0
  %24 = zext i1 %23 to i8
  %25 = add i8 %22, %24
  store i8 %25, i8* %20, align 1, !nosanitize !0
  %new_r_abs.i.i = add i128 %new_r_add.i.i, %read.in.input
  %is_zero3.i.i = icmp eq i128 %new_r_abs.i.i, 0
  br i1 %is_zero3.i.i, label %compute_inverse.i.error.i.i_crit_edge, label %compute_inverse.i.continue.i.i_crit_edge

compute_inverse.i.continue.i.i_crit_edge:         ; preds = %compute_inverse.i
  br label %continue.i.i

compute_inverse.i.error.i.i_crit_edge:            ; preds = %compute_inverse.i
  %26 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([18 x i32]* @__sancov_gen_.9 to i64), i64 8) to i32*), align 4, !nosanitize !0
  %27 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %28 = getelementptr i8, i8* %27, i32 %26
  %29 = load i8, i8* %28, align 1, !nosanitize !0
  %30 = add i8 %29, 1
  %31 = icmp eq i8 %30, 0
  %32 = zext i1 %31 to i8
  %33 = add i8 %30, %32
  store i8 %33, i8* %28, align 1, !nosanitize !0
  br label %error.i.i

continue.i.i:                                     ; preds = %continue.i.i.continue.i.i_crit_edge, %compute_inverse.i.continue.i.i_crit_edge
  %new_r.07.i.i = phi i128 [ %34, %continue.i.i.continue.i.i_crit_edge ], [ %new_r_abs.i.i, %compute_inverse.i.continue.i.i_crit_edge ]
  %r.06.i.i = phi i128 [ %new_r.07.i.i, %continue.i.i.continue.i.i_crit_edge ], [ 9938766679346745377, %compute_inverse.i.continue.i.i_crit_edge ]
  %34 = srem i128 %r.06.i.i, %new_r.07.i.i
  %is_zero.i.i = icmp eq i128 %34, 0
  br i1 %is_zero.i.i, label %end.i.i, label %continue.i.i.continue.i.i_crit_edge

continue.i.i.continue.i.i_crit_edge:              ; preds = %continue.i.i
  %35 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([18 x i32]* @__sancov_gen_.9 to i64), i64 12) to i32*), align 4, !nosanitize !0
  %36 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %37 = getelementptr i8, i8* %36, i32 %35
  %38 = load i8, i8* %37, align 1, !nosanitize !0
  %39 = add i8 %38, 1
  %40 = icmp eq i8 %39, 0
  %41 = zext i1 %40 to i8
  %42 = add i8 %39, %41
  store i8 %42, i8* %37, align 1, !nosanitize !0
  br label %continue.i.i

end.i.i:                                          ; preds = %continue.i.i
  %inverse_exists.i.i = icmp eq i128 %new_r.07.i.i, 1
  br i1 %inverse_exists.i.i, label %end.i.i.mod_div.exit_crit_edge, label %end.i.i.error.i.i_crit_edge

end.i.i.error.i.i_crit_edge:                      ; preds = %end.i.i
  %43 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([18 x i32]* @__sancov_gen_.9 to i64), i64 16) to i32*), align 4, !nosanitize !0
  %44 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %45 = getelementptr i8, i8* %44, i32 %43
  %46 = load i8, i8* %45, align 1, !nosanitize !0
  %47 = add i8 %46, 1
  %48 = icmp eq i8 %47, 0
  %49 = zext i1 %48 to i8
  %50 = add i8 %47, %49
  store i8 %50, i8* %45, align 1, !nosanitize !0
  br label %error.i.i

end.i.i.mod_div.exit_crit_edge:                   ; preds = %end.i.i
  %51 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([18 x i32]* @__sancov_gen_.9 to i64), i64 20) to i32*), align 4, !nosanitize !0
  %52 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %53 = getelementptr i8, i8* %52, i32 %51
  %54 = load i8, i8* %53, align 1, !nosanitize !0
  %55 = add i8 %54, 1
  %56 = icmp eq i8 %55, 0
  %57 = zext i1 %56 to i8
  %58 = add i8 %55, %57
  store i8 %58, i8* %53, align 1, !nosanitize !0
  br label %mod_div.exit

error.i.i:                                        ; preds = %end.i.i.error.i.i_crit_edge, %compute_inverse.i.error.i.i_crit_edge
  tail call void @llvm.trap() #6
  unreachable

mod_div.exit:                                     ; preds = %end.i.i.mod_div.exit_crit_edge, %entry.mod_div.exit_crit_edge
  %sub.i = sub i128 0, %read.in.input
  %result.i = srem i128 %sub.i, 9938766679346745377
  %is_neg.i = icmp slt i128 %result.i, 0
  %pos_result.i = select i1 %is_neg.i, i128 9938766679346745377, i128 0
  %59 = select i1 %is_neg.i, i32* inttoptr (i64 add (i64 ptrtoint ([18 x i32]* @__sancov_gen_.9 to i64), i64 32) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([18 x i32]* @__sancov_gen_.9 to i64), i64 36) to i32*)
  %60 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %61 = load i32, i32* %59, align 4, !nosanitize !0
  %62 = getelementptr i8, i8* %60, i32 %61
  %63 = load i8, i8* %62, align 1, !nosanitize !0
  %64 = add i8 %63, 1
  %65 = icmp eq i8 %64, 0
  %66 = zext i1 %65 to i8
  %67 = add i8 %64, %66
  store i8 %67, i8* %62, align 1, !nosanitize !0
  %final_result.i = add nsw i128 %pos_result.i, %result.i
  %prod.i = mul i128 %final_result.i, %free.read.inv.inter
  %result.i7 = srem i128 %prod.i, 9938766679346745377
  %is_neg.i8 = icmp slt i128 %result.i7, 0
  %pos_result.i9 = select i1 %is_neg.i8, i128 9938766679346745377, i128 0
  %68 = select i1 %is_neg.i8, i32* inttoptr (i64 add (i64 ptrtoint ([18 x i32]* @__sancov_gen_.9 to i64), i64 40) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([18 x i32]* @__sancov_gen_.9 to i64), i64 44) to i32*)
  %69 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %70 = load i32, i32* %68, align 4, !nosanitize !0
  %71 = getelementptr i8, i8* %69, i32 %70
  %72 = load i8, i8* %71, align 1, !nosanitize !0
  %73 = add i8 %72, 1
  %74 = icmp eq i8 %73, 0
  %75 = zext i1 %74 to i8
  %76 = add i8 %73, %75
  store i8 %76, i8* %71, align 1, !nosanitize !0
  %final_result.i10 = add nsw i128 %result.i7, 1
  %add.i = add nsw i128 %final_result.i10, %pos_result.i9
  %result.i11 = srem i128 %add.i, 9938766679346745377
  %is_neg.i12 = icmp slt i128 %result.i11, 0
  %pos_result.i13 = select i1 %is_neg.i12, i128 9938766679346745377, i128 0
  %77 = select i1 %is_neg.i12, i32* inttoptr (i64 add (i64 ptrtoint ([18 x i32]* @__sancov_gen_.9 to i64), i64 48) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([18 x i32]* @__sancov_gen_.9 to i64), i64 52) to i32*)
  %78 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %79 = load i32, i32* %77, align 4, !nosanitize !0
  %80 = getelementptr i8, i8* %78, i32 %79
  %81 = load i8, i8* %80, align 1, !nosanitize !0
  %82 = add i8 %81, 1
  %83 = icmp eq i8 %82, 0
  %84 = zext i1 %83 to i8
  %85 = add i8 %82, %84
  store i8 %85, i8* %80, align 1, !nosanitize !0
  %final_result.i14 = add nsw i128 %pos_result.i13, %result.i11
  store i1 true, i1* @constraint, align 1
  %sub.i15 = add nsw i128 %final_result.i14, -1
  %result.i16 = srem i128 %sub.i15, 9938766679346745377
  %is_neg.i17 = icmp slt i128 %result.i16, 0
  %pos_result.i18 = select i1 %is_neg.i17, i128 9938766679346745377, i128 0
  %86 = select i1 %is_neg.i17, i32* inttoptr (i64 add (i64 ptrtoint ([18 x i32]* @__sancov_gen_.9 to i64), i64 56) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([18 x i32]* @__sancov_gen_.9 to i64), i64 60) to i32*)
  %87 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %88 = load i32, i32* %86, align 4, !nosanitize !0
  %89 = getelementptr i8, i8* %87, i32 %88
  %90 = load i8, i8* %89, align 1, !nosanitize !0
  %91 = add i8 %90, 1
  %92 = icmp eq i8 %91, 0
  %93 = zext i1 %92 to i8
  %94 = add i8 %91, %93
  store i8 %94, i8* %89, align 1, !nosanitize !0
  %final_result.i19 = add nsw i128 %pos_result.i18, %result.i16
  %prod.i20 = mul i128 %final_result.i19, %final_result.i14
  %result.i21 = srem i128 %prod.i20, 9938766679346745377
  %is_neg.i22 = icmp slt i128 %result.i21, 0
  %pos_result.i23 = select i1 %is_neg.i22, i128 9938766679346745377, i128 0
  %95 = select i1 %is_neg.i22, i32* inttoptr (i64 add (i64 ptrtoint ([18 x i32]* @__sancov_gen_.9 to i64), i64 64) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([18 x i32]* @__sancov_gen_.9 to i64), i64 68) to i32*)
  %96 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %97 = load i32, i32* %95, align 4, !nosanitize !0
  %98 = getelementptr i8, i8* %96, i32 %97
  %99 = load i8, i8* %98, align 1, !nosanitize !0
  %100 = add i8 %99, 1
  %101 = icmp eq i8 %100, 0
  %102 = zext i1 %101 to i8
  %103 = add i8 %100, %102
  store i8 %103, i8* %98, align 1, !nosanitize !0
  %final_result.i24 = sub nsw i128 0, %result.i21
  %constraint.i = icmp eq i128 %pos_result.i23, %final_result.i24
  store i1 %constraint.i, i1* @constraint.1, align 1
  %"gep.IsZero|out.output" = getelementptr inbounds %struct_template_IsZero, %struct_template_IsZero* %0, i64 0, i32 2
  store i128 %final_result.i14, i128* %"gep.IsZero|out.output", align 4
  ret void
}

; Function Attrs: nounwind
define i32 @main() local_unnamed_addr #6 comdat {
entry:
  %0 = alloca i64, align 8
  %1 = alloca i64, align 8
  %2 = alloca i64, align 8
  %3 = alloca i64, align 8
  %4 = load i32, i32* getelementptr inbounds ([39 x i32], [39 x i32]* @__sancov_gen_.10, i32 0, i32 0), align 4, !nosanitize !0
  %5 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %6 = getelementptr i8, i8* %5, i32 %4
  %7 = load i8, i8* %6, align 1, !nosanitize !0
  %8 = add i8 %7, 1
  %9 = icmp eq i8 %8, 0
  %10 = zext i1 %9 to i8
  %11 = add i8 %8, %10
  store i8 %11, i8* %6, align 1, !nosanitize !0
  %12 = call i32 (i8*, ...) @scanf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.map.lld, i64 0, i64 0), i64* nonnull %0)
  %13 = call i32 (i8*, ...) @scanf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.map.lld, i64 0, i64 0), i64* nonnull %1)
  %14 = load i64, i64* %0, align 8
  %15 = load i64, i64* %1, align 8
  %16 = zext i64 %14 to i128
  %17 = zext i64 %15 to i128
  %18 = shl nuw i128 %17, 64
  %19 = or i128 %18, %16
  %20 = call i32 (i8*, ...) @scanf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.map.lld, i64 0, i64 0), i64* nonnull %2)
  %21 = call i32 (i8*, ...) @scanf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.map.lld, i64 0, i64 0), i64* nonnull %3)
  %22 = load i64, i64* %2, align 8
  %23 = load i64, i64* %3, align 8
  %24 = zext i64 %22 to i128
  %25 = zext i64 %23 to i128
  %26 = shl nuw i128 %25, 64
  %27 = or i128 %26, %24
  %b_is_zero.i.i = icmp eq i128 %19, 0
  br i1 %b_is_zero.i.i, label %entry.cloned_fn_template_init_IsZero.exit_crit_edge, label %compute_inverse.i.i

entry.cloned_fn_template_init_IsZero.exit_crit_edge: ; preds = %entry
  %28 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 4) to i32*), align 4, !nosanitize !0
  %29 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %30 = getelementptr i8, i8* %29, i32 %28
  %31 = load i8, i8* %30, align 1, !nosanitize !0
  %32 = add i8 %31, 1
  %33 = icmp eq i8 %32, 0
  %34 = zext i1 %33 to i8
  %35 = add i8 %32, %34
  store i8 %35, i8* %30, align 1, !nosanitize !0
  br label %cloned_fn_template_init_IsZero.exit

compute_inverse.i.i:                              ; preds = %entry
  %new_r_neg.i.i.i = icmp slt i128 %19, 0
  %new_r_add.i.i.i = select i1 %new_r_neg.i.i.i, i128 9938766679346745377, i128 0
  %36 = select i1 %new_r_neg.i.i.i, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 52) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 56) to i32*)
  %37 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %38 = load i32, i32* %36, align 4, !nosanitize !0
  %39 = getelementptr i8, i8* %37, i32 %38
  %40 = load i8, i8* %39, align 1, !nosanitize !0
  %41 = add i8 %40, 1
  %42 = icmp eq i8 %41, 0
  %43 = zext i1 %42 to i8
  %44 = add i8 %41, %43
  store i8 %44, i8* %39, align 1, !nosanitize !0
  %new_r_abs.i.i.i = add i128 %new_r_add.i.i.i, %19
  %is_zero3.i.i.i = icmp eq i128 %new_r_abs.i.i.i, 0
  br i1 %is_zero3.i.i.i, label %compute_inverse.i.i.error.i.i.i_crit_edge, label %compute_inverse.i.i.continue.i.i.i_crit_edge

compute_inverse.i.i.continue.i.i.i_crit_edge:     ; preds = %compute_inverse.i.i
  br label %continue.i.i.i

compute_inverse.i.i.error.i.i.i_crit_edge:        ; preds = %compute_inverse.i.i
  %45 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 8) to i32*), align 4, !nosanitize !0
  %46 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %47 = getelementptr i8, i8* %46, i32 %45
  %48 = load i8, i8* %47, align 1, !nosanitize !0
  %49 = add i8 %48, 1
  %50 = icmp eq i8 %49, 0
  %51 = zext i1 %50 to i8
  %52 = add i8 %49, %51
  store i8 %52, i8* %47, align 1, !nosanitize !0
  br label %error.i.i.i

continue.i.i.i:                                   ; preds = %continue.i.i.i.continue.i.i.i_crit_edge, %compute_inverse.i.i.continue.i.i.i_crit_edge
  %new_r.07.i.i.i = phi i128 [ %53, %continue.i.i.i.continue.i.i.i_crit_edge ], [ %new_r_abs.i.i.i, %compute_inverse.i.i.continue.i.i.i_crit_edge ]
  %r.06.i.i.i = phi i128 [ %new_r.07.i.i.i, %continue.i.i.i.continue.i.i.i_crit_edge ], [ 9938766679346745377, %compute_inverse.i.i.continue.i.i.i_crit_edge ]
  %53 = srem i128 %r.06.i.i.i, %new_r.07.i.i.i
  %is_zero.i.i.i = icmp eq i128 %53, 0
  br i1 %is_zero.i.i.i, label %end.i.i.i, label %continue.i.i.i.continue.i.i.i_crit_edge

continue.i.i.i.continue.i.i.i_crit_edge:          ; preds = %continue.i.i.i
  %54 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 12) to i32*), align 4, !nosanitize !0
  %55 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %56 = getelementptr i8, i8* %55, i32 %54
  %57 = load i8, i8* %56, align 1, !nosanitize !0
  %58 = add i8 %57, 1
  %59 = icmp eq i8 %58, 0
  %60 = zext i1 %59 to i8
  %61 = add i8 %58, %60
  store i8 %61, i8* %56, align 1, !nosanitize !0
  br label %continue.i.i.i

end.i.i.i:                                        ; preds = %continue.i.i.i
  %inverse_exists.i.i.i = icmp eq i128 %new_r.07.i.i.i, 1
  br i1 %inverse_exists.i.i.i, label %end.i.i.i.cloned_fn_template_init_IsZero.exit_crit_edge, label %end.i.i.i.error.i.i.i_crit_edge

end.i.i.i.error.i.i.i_crit_edge:                  ; preds = %end.i.i.i
  %62 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 16) to i32*), align 4, !nosanitize !0
  %63 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %64 = getelementptr i8, i8* %63, i32 %62
  %65 = load i8, i8* %64, align 1, !nosanitize !0
  %66 = add i8 %65, 1
  %67 = icmp eq i8 %66, 0
  %68 = zext i1 %67 to i8
  %69 = add i8 %66, %68
  store i8 %69, i8* %64, align 1, !nosanitize !0
  br label %error.i.i.i

end.i.i.i.cloned_fn_template_init_IsZero.exit_crit_edge: ; preds = %end.i.i.i
  %70 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 20) to i32*), align 4, !nosanitize !0
  %71 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %72 = getelementptr i8, i8* %71, i32 %70
  %73 = load i8, i8* %72, align 1, !nosanitize !0
  %74 = add i8 %73, 1
  %75 = icmp eq i8 %74, 0
  %76 = zext i1 %75 to i8
  %77 = add i8 %74, %76
  store i8 %77, i8* %72, align 1, !nosanitize !0
  br label %cloned_fn_template_init_IsZero.exit

error.i.i.i:                                      ; preds = %end.i.i.i.error.i.i.i_crit_edge, %compute_inverse.i.i.error.i.i.i_crit_edge
  call void @llvm.trap() #6
  unreachable

cloned_fn_template_init_IsZero.exit:              ; preds = %end.i.i.i.cloned_fn_template_init_IsZero.exit_crit_edge, %entry.cloned_fn_template_init_IsZero.exit_crit_edge
  %sub.i.i = sub i128 0, %19
  %result.i.i = srem i128 %sub.i.i, 9938766679346745377
  %is_neg.i.i = icmp slt i128 %result.i.i, 0
  %pos_result.i.i = select i1 %is_neg.i.i, i128 9938766679346745377, i128 0
  %78 = select i1 %is_neg.i.i, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 60) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 64) to i32*)
  %79 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %80 = load i32, i32* %78, align 4, !nosanitize !0
  %81 = getelementptr i8, i8* %79, i32 %80
  %82 = load i8, i8* %81, align 1, !nosanitize !0
  %83 = add i8 %82, 1
  %84 = icmp eq i8 %83, 0
  %85 = zext i1 %84 to i8
  %86 = add i8 %83, %85
  store i8 %86, i8* %81, align 1, !nosanitize !0
  %final_result.i.i = add nsw i128 %pos_result.i.i, %result.i.i
  %prod.i.i = mul i128 %27, %final_result.i.i
  %result.i7.i = srem i128 %prod.i.i, 9938766679346745377
  %is_neg.i8.i = icmp slt i128 %result.i7.i, 0
  %pos_result.i9.i = select i1 %is_neg.i8.i, i128 9938766679346745377, i128 0
  %87 = select i1 %is_neg.i8.i, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 68) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 72) to i32*)
  %88 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %89 = load i32, i32* %87, align 4, !nosanitize !0
  %90 = getelementptr i8, i8* %88, i32 %89
  %91 = load i8, i8* %90, align 1, !nosanitize !0
  %92 = add i8 %91, 1
  %93 = icmp eq i8 %92, 0
  %94 = zext i1 %93 to i8
  %95 = add i8 %92, %94
  store i8 %95, i8* %90, align 1, !nosanitize !0
  %final_result.i10.i = add nsw i128 %result.i7.i, 1
  %add.i.i = add nsw i128 %final_result.i10.i, %pos_result.i9.i
  %result.i11.i = srem i128 %add.i.i, 9938766679346745377
  %is_neg.i12.i = icmp slt i128 %result.i11.i, 0
  %pos_result.i13.i = select i1 %is_neg.i12.i, i128 9938766679346745377, i128 0
  %96 = select i1 %is_neg.i12.i, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 76) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 80) to i32*)
  %97 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %98 = load i32, i32* %96, align 4, !nosanitize !0
  %99 = getelementptr i8, i8* %97, i32 %98
  %100 = load i8, i8* %99, align 1, !nosanitize !0
  %101 = add i8 %100, 1
  %102 = icmp eq i8 %101, 0
  %103 = zext i1 %102 to i8
  %104 = add i8 %101, %103
  store i8 %104, i8* %99, align 1, !nosanitize !0
  %final_result.i14.i = add nsw i128 %pos_result.i13.i, %result.i11.i
  store i1 true, i1* @constraint, align 1
  %sub.i15.i = add nsw i128 %final_result.i14.i, -1
  %result.i16.i = srem i128 %sub.i15.i, 9938766679346745377
  %is_neg.i17.i = icmp slt i128 %result.i16.i, 0
  %pos_result.i18.i = select i1 %is_neg.i17.i, i128 9938766679346745377, i128 0
  %105 = select i1 %is_neg.i17.i, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 84) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 88) to i32*)
  %106 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %107 = load i32, i32* %105, align 4, !nosanitize !0
  %108 = getelementptr i8, i8* %106, i32 %107
  %109 = load i8, i8* %108, align 1, !nosanitize !0
  %110 = add i8 %109, 1
  %111 = icmp eq i8 %110, 0
  %112 = zext i1 %111 to i8
  %113 = add i8 %110, %112
  store i8 %113, i8* %108, align 1, !nosanitize !0
  %final_result.i19.i = add nsw i128 %pos_result.i18.i, %result.i16.i
  %prod.i20.i = mul i128 %final_result.i19.i, %final_result.i14.i
  %result.i21.i = srem i128 %prod.i20.i, 9938766679346745377
  %is_neg.i22.i = icmp slt i128 %result.i21.i, 0
  %pos_result.i23.i = select i1 %is_neg.i22.i, i128 9938766679346745377, i128 0
  %114 = select i1 %is_neg.i22.i, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 92) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 96) to i32*)
  %115 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %116 = load i32, i32* %114, align 4, !nosanitize !0
  %117 = getelementptr i8, i8* %115, i32 %116
  %118 = load i8, i8* %117, align 1, !nosanitize !0
  %119 = add i8 %118, 1
  %120 = icmp eq i8 %119, 0
  %121 = zext i1 %120 to i8
  %122 = add i8 %119, %121
  store i8 %122, i8* %117, align 1, !nosanitize !0
  %final_result.i24.i = sub nsw i128 0, %result.i21.i
  %constraint.i.i = icmp eq i128 %pos_result.i23.i, %final_result.i24.i
  store i1 %constraint.i.i, i1* @constraint.1, align 1
  %123 = trunc i128 %final_result.i14.i to i64
  %124 = lshr i128 %final_result.i14.i, 64
  %125 = trunc i128 %124 to i64
  %126 = call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([5 x i8], [5 x i8]* @.str.map.ld, i64 0, i64 0), i64 %123)
  %127 = call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([5 x i8], [5 x i8]* @.str.map.ld, i64 0, i64 0), i64 %125)
  %128 = load i1, i1* @constraint, align 1
  %129 = load i1, i1* @constraint.1, align 1
  %130 = and i1 %128, %129
  %131 = call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.map.d, i64 0, i64 0), i1 %130)
  br i1 %b_is_zero.i.i, label %cloned_fn_template_init_IsZero.exit.fn_template_init_IsZero.exit_crit_edge, label %compute_inverse.i.i10

cloned_fn_template_init_IsZero.exit.fn_template_init_IsZero.exit_crit_edge: ; preds = %cloned_fn_template_init_IsZero.exit
  %132 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 24) to i32*), align 4, !nosanitize !0
  %133 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %134 = getelementptr i8, i8* %133, i32 %132
  %135 = load i8, i8* %134, align 1, !nosanitize !0
  %136 = add i8 %135, 1
  %137 = icmp eq i8 %136, 0
  %138 = zext i1 %137 to i8
  %139 = add i8 %136, %138
  store i8 %139, i8* %134, align 1, !nosanitize !0
  br label %fn_template_init_IsZero.exit

compute_inverse.i.i10:                            ; preds = %cloned_fn_template_init_IsZero.exit
  %new_r_neg.i.i.i6 = icmp slt i128 %19, 0
  %new_r_add.i.i.i7 = select i1 %new_r_neg.i.i.i6, i128 9938766679346745377, i128 0
  %140 = select i1 %new_r_neg.i.i.i6, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 100) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 104) to i32*)
  %141 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %142 = load i32, i32* %140, align 4, !nosanitize !0
  %143 = getelementptr i8, i8* %141, i32 %142
  %144 = load i8, i8* %143, align 1, !nosanitize !0
  %145 = add i8 %144, 1
  %146 = icmp eq i8 %145, 0
  %147 = zext i1 %146 to i8
  %148 = add i8 %145, %147
  store i8 %148, i8* %143, align 1, !nosanitize !0
  %new_r_abs.i.i.i8 = add i128 %new_r_add.i.i.i7, %19
  %is_zero3.i.i.i9 = icmp eq i128 %new_r_abs.i.i.i8, 0
  br i1 %is_zero3.i.i.i9, label %compute_inverse.i.i10.error.i.i.i17_crit_edge, label %compute_inverse.i.i10.continue.i.i.i14_crit_edge

compute_inverse.i.i10.continue.i.i.i14_crit_edge: ; preds = %compute_inverse.i.i10
  br label %continue.i.i.i14

compute_inverse.i.i10.error.i.i.i17_crit_edge:    ; preds = %compute_inverse.i.i10
  %149 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 28) to i32*), align 4, !nosanitize !0
  %150 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %151 = getelementptr i8, i8* %150, i32 %149
  %152 = load i8, i8* %151, align 1, !nosanitize !0
  %153 = add i8 %152, 1
  %154 = icmp eq i8 %153, 0
  %155 = zext i1 %154 to i8
  %156 = add i8 %153, %155
  store i8 %156, i8* %151, align 1, !nosanitize !0
  br label %error.i.i.i17

continue.i.i.i14:                                 ; preds = %continue.i.i.i14.continue.i.i.i14_crit_edge, %compute_inverse.i.i10.continue.i.i.i14_crit_edge
  %new_r.07.i.i.i11 = phi i128 [ %new_r_updated.i.i.i, %continue.i.i.i14.continue.i.i.i14_crit_edge ], [ %new_r_abs.i.i.i8, %compute_inverse.i.i10.continue.i.i.i14_crit_edge ]
  %r.06.i.i.i12 = phi i128 [ %new_r.07.i.i.i11, %continue.i.i.i14.continue.i.i.i14_crit_edge ], [ 9938766679346745377, %compute_inverse.i.i10.continue.i.i.i14_crit_edge ]
  %new_t.05.i.i.i = phi i128 [ %new_t_updated.i.i.i, %continue.i.i.i14.continue.i.i.i14_crit_edge ], [ 1, %compute_inverse.i.i10.continue.i.i.i14_crit_edge ]
  %t.04.i.i.i = phi i128 [ %new_t.05.i.i.i, %continue.i.i.i14.continue.i.i.i14_crit_edge ], [ 0, %compute_inverse.i.i10.continue.i.i.i14_crit_edge ]
  %quotient.i.i.i = sdiv i128 %r.06.i.i.i12, %new_r.07.i.i.i11
  %temp_t.i.i.i = mul i128 %quotient.i.i.i, %new_t.05.i.i.i
  %new_t_updated.i.i.i = sub i128 %t.04.i.i.i, %temp_t.i.i.i
  %temp_r.i.i.i = mul i128 %quotient.i.i.i, %new_r.07.i.i.i11
  %new_r_updated.i.i.i = sub i128 %r.06.i.i.i12, %temp_r.i.i.i
  %is_zero.i.i.i13 = icmp eq i128 %new_r_updated.i.i.i, 0
  br i1 %is_zero.i.i.i13, label %end.i.i.i16, label %continue.i.i.i14.continue.i.i.i14_crit_edge

continue.i.i.i14.continue.i.i.i14_crit_edge:      ; preds = %continue.i.i.i14
  %157 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 32) to i32*), align 4, !nosanitize !0
  %158 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %159 = getelementptr i8, i8* %158, i32 %157
  %160 = load i8, i8* %159, align 1, !nosanitize !0
  %161 = add i8 %160, 1
  %162 = icmp eq i8 %161, 0
  %163 = zext i1 %162 to i8
  %164 = add i8 %161, %163
  store i8 %164, i8* %159, align 1, !nosanitize !0
  br label %continue.i.i.i14

end.i.i.i16:                                      ; preds = %continue.i.i.i14
  %inverse_exists.i.i.i15 = icmp eq i128 %new_r.07.i.i.i11, 1
  br i1 %inverse_exists.i.i.i15, label %mod_inverse.exit.i.i, label %end.i.i.i16.error.i.i.i17_crit_edge

end.i.i.i16.error.i.i.i17_crit_edge:              ; preds = %end.i.i.i16
  %165 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 36) to i32*), align 4, !nosanitize !0
  %166 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %167 = getelementptr i8, i8* %166, i32 %165
  %168 = load i8, i8* %167, align 1, !nosanitize !0
  %169 = add i8 %168, 1
  %170 = icmp eq i8 %169, 0
  %171 = zext i1 %170 to i8
  %172 = add i8 %169, %171
  store i8 %172, i8* %167, align 1, !nosanitize !0
  br label %error.i.i.i17

error.i.i.i17:                                    ; preds = %end.i.i.i16.error.i.i.i17_crit_edge, %compute_inverse.i.i10.error.i.i.i17_crit_edge
  call void @llvm.trap() #6
  unreachable

mod_inverse.exit.i.i:                             ; preds = %end.i.i.i16
  %173 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 40) to i32*), align 4, !nosanitize !0
  %174 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %175 = getelementptr i8, i8* %174, i32 %173
  %176 = load i8, i8* %175, align 1, !nosanitize !0
  %177 = add i8 %176, 1
  %178 = icmp eq i8 %177, 0
  %179 = zext i1 %178 to i8
  %180 = add i8 %177, %179
  store i8 %180, i8* %175, align 1, !nosanitize !0
  %result_neg.i.i.i = icmp slt i128 %new_t.05.i.i.i, 0
  %result_add.i.i.i = select i1 %result_neg.i.i.i, i128 9938766679346745377, i128 0
  %181 = select i1 %result_neg.i.i.i, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 108) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 112) to i32*)
  %182 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %183 = load i32, i32* %181, align 4, !nosanitize !0
  %184 = getelementptr i8, i8* %182, i32 %183
  %185 = load i8, i8* %184, align 1, !nosanitize !0
  %186 = add i8 %185, 1
  %187 = icmp eq i8 %186, 0
  %188 = zext i1 %187 to i8
  %189 = add i8 %186, %188
  store i8 %189, i8* %184, align 1, !nosanitize !0
  %result_pos.i.i.i = add i128 %result_add.i.i.i, %new_t.05.i.i.i
  %final_result.i.i.i = srem i128 %result_pos.i.i.i, 9938766679346745377
  %is_neg.i.i.i = icmp slt i128 %final_result.i.i.i, 0
  %pos_result.i.i.i = select i1 %is_neg.i.i.i, i128 9938766679346745377, i128 0
  %190 = select i1 %is_neg.i.i.i, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 116) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 120) to i32*)
  %191 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %192 = load i32, i32* %190, align 4, !nosanitize !0
  %193 = getelementptr i8, i8* %191, i32 %192
  %194 = load i8, i8* %193, align 1, !nosanitize !0
  %195 = add i8 %194, 1
  %196 = icmp eq i8 %195, 0
  %197 = zext i1 %196 to i8
  %198 = add i8 %195, %197
  store i8 %198, i8* %193, align 1, !nosanitize !0
  %final_result.i1.i.i = add nsw i128 %pos_result.i.i.i, %final_result.i.i.i
  br label %fn_template_init_IsZero.exit

fn_template_init_IsZero.exit:                     ; preds = %cloned_fn_template_init_IsZero.exit.fn_template_init_IsZero.exit_crit_edge, %mod_inverse.exit.i.i
  %..i.i = phi i128 [ 0, %cloned_fn_template_init_IsZero.exit.fn_template_init_IsZero.exit_crit_edge ], [ %final_result.i1.i.i, %mod_inverse.exit.i.i ]
  %prod.i.i23 = mul i128 %..i.i, %final_result.i.i
  %result.i7.i24 = srem i128 %prod.i.i23, 9938766679346745377
  %is_neg.i8.i25 = icmp slt i128 %result.i7.i24, 0
  %pos_result.i9.i26 = select i1 %is_neg.i8.i25, i128 9938766679346745377, i128 0
  %199 = select i1 %is_neg.i8.i25, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 124) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 128) to i32*)
  %200 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %201 = load i32, i32* %199, align 4, !nosanitize !0
  %202 = getelementptr i8, i8* %200, i32 %201
  %203 = load i8, i8* %202, align 1, !nosanitize !0
  %204 = add i8 %203, 1
  %205 = icmp eq i8 %204, 0
  %206 = zext i1 %205 to i8
  %207 = add i8 %204, %206
  store i8 %207, i8* %202, align 1, !nosanitize !0
  %final_result.i10.i27 = add nsw i128 %result.i7.i24, 1
  %add.i.i28 = add nsw i128 %final_result.i10.i27, %pos_result.i9.i26
  %result.i11.i29 = srem i128 %add.i.i28, 9938766679346745377
  %is_neg.i12.i30 = icmp slt i128 %result.i11.i29, 0
  %pos_result.i13.i31 = select i1 %is_neg.i12.i30, i128 9938766679346745377, i128 0
  %208 = select i1 %is_neg.i12.i30, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 132) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 136) to i32*)
  %209 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %210 = load i32, i32* %208, align 4, !nosanitize !0
  %211 = getelementptr i8, i8* %209, i32 %210
  %212 = load i8, i8* %211, align 1, !nosanitize !0
  %213 = add i8 %212, 1
  %214 = icmp eq i8 %213, 0
  %215 = zext i1 %214 to i8
  %216 = add i8 %213, %215
  store i8 %216, i8* %211, align 1, !nosanitize !0
  %final_result.i14.i32 = add nsw i128 %pos_result.i13.i31, %result.i11.i29
  store i1 true, i1* @constraint, align 1
  %sub.i15.i33 = add nsw i128 %final_result.i14.i32, -1
  %result.i16.i34 = srem i128 %sub.i15.i33, 9938766679346745377
  %is_neg.i17.i35 = icmp slt i128 %result.i16.i34, 0
  %pos_result.i18.i36 = select i1 %is_neg.i17.i35, i128 9938766679346745377, i128 0
  %217 = select i1 %is_neg.i17.i35, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 140) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 144) to i32*)
  %218 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %219 = load i32, i32* %217, align 4, !nosanitize !0
  %220 = getelementptr i8, i8* %218, i32 %219
  %221 = load i8, i8* %220, align 1, !nosanitize !0
  %222 = add i8 %221, 1
  %223 = icmp eq i8 %222, 0
  %224 = zext i1 %223 to i8
  %225 = add i8 %222, %224
  store i8 %225, i8* %220, align 1, !nosanitize !0
  %final_result.i19.i37 = add nsw i128 %pos_result.i18.i36, %result.i16.i34
  %prod.i20.i38 = mul i128 %final_result.i19.i37, %final_result.i14.i32
  %result.i21.i39 = srem i128 %prod.i20.i38, 9938766679346745377
  %is_neg.i22.i40 = icmp slt i128 %result.i21.i39, 0
  %pos_result.i23.i41 = select i1 %is_neg.i22.i40, i128 9938766679346745377, i128 0
  %226 = select i1 %is_neg.i22.i40, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 148) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 152) to i32*)
  %227 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %228 = load i32, i32* %226, align 4, !nosanitize !0
  %229 = getelementptr i8, i8* %227, i32 %228
  %230 = load i8, i8* %229, align 1, !nosanitize !0
  %231 = add i8 %230, 1
  %232 = icmp eq i8 %231, 0
  %233 = zext i1 %232 to i8
  %234 = add i8 %231, %233
  store i8 %234, i8* %229, align 1, !nosanitize !0
  %final_result.i24.i42 = sub nsw i128 0, %result.i21.i39
  %constraint.i.i43 = icmp eq i128 %pos_result.i23.i41, %final_result.i24.i42
  store i1 %constraint.i.i43, i1* @constraint.1, align 1
  %235 = trunc i128 %final_result.i14.i32 to i64
  %236 = lshr i128 %final_result.i14.i32, 64
  %237 = trunc i128 %236 to i64
  %238 = call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([5 x i8], [5 x i8]* @.str.map.ld, i64 0, i64 0), i64 %235)
  %239 = call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([5 x i8], [5 x i8]* @.str.map.ld, i64 0, i64 0), i64 %237)
  %240 = load i1, i1* @constraint, align 1
  %241 = load i1, i1* @constraint.1, align 1
  %242 = and i1 %240, %241
  %243 = call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.map.d, i64 0, i64 0), i1 %242)
  %outputNotEqual = icmp ne i128 %final_result.i14.i, %final_result.i14.i32
  %tmp_under_constrained_condition = and i1 %242, %outputNotEqual
  %final_under_constrained_condition = and i1 %130, %tmp_under_constrained_condition
  br i1 %final_under_constrained_condition, label %under_constrained_error, label %no_under_constrained_continue

under_constrained_error:                          ; preds = %fn_template_init_IsZero.exit
  %244 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 44) to i32*), align 4, !nosanitize !0
  %245 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %246 = getelementptr i8, i8* %245, i32 %244
  %247 = load i8, i8* %246, align 1, !nosanitize !0
  %248 = add i8 %247, 1
  %249 = icmp eq i8 %248, 0
  %250 = zext i1 %249 to i8
  %251 = add i8 %248, %250
  store i8 %251, i8* %246, align 1, !nosanitize !0
  %puts = call i32 @puts(i8* nonnull dereferenceable(1) getelementptr inbounds ([60 x i8], [60 x i8]* @str, i64 0, i64 0))
  call void @llvm.trap()
  unreachable

no_under_constrained_continue:                    ; preds = %fn_template_init_IsZero.exit
  %252 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([39 x i32]* @__sancov_gen_.10 to i64), i64 48) to i32*), align 4, !nosanitize !0
  %253 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %254 = getelementptr i8, i8* %253, i32 %252
  %255 = load i8, i8* %254, align 1, !nosanitize !0
  %256 = add i8 %255, 1
  %257 = icmp eq i8 %256, 0
  %258 = zext i1 %257 to i8
  %259 = add i8 %256, %258
  store i8 %259, i8* %254, align 1, !nosanitize !0
  ret i32 0
}

; Function Attrs: nofree nounwind
declare noundef i32 @scanf(i8* nocapture noundef readonly, ...) local_unnamed_addr #7

; Function Attrs: nofree nounwind
declare noundef i32 @printf(i8* nocapture noundef readonly, ...) local_unnamed_addr #7

; Function Attrs: cold noreturn nounwind
declare void @llvm.trap() #8

; Function Attrs: mustprogress nofree norecurse nosync nounwind readnone willreturn
define i128 @mod_add(i128 %a, i128 %b, i128 %m) local_unnamed_addr #1 comdat {
entry:
  %0 = load i32, i32* getelementptr inbounds ([3 x i32], [3 x i32]* @__sancov_gen_.11, i32 0, i32 0), align 4, !nosanitize !0
  %1 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %2 = getelementptr i8, i8* %1, i32 %0
  %3 = load i8, i8* %2, align 1, !nosanitize !0
  %4 = add i8 %3, 1
  %5 = icmp eq i8 %4, 0
  %6 = zext i1 %5 to i8
  %7 = add i8 %4, %6
  store i8 %7, i8* %2, align 1, !nosanitize !0
  %add = add i128 %b, %a
  %result = srem i128 %add, %m
  %is_neg = icmp slt i128 %result, 0
  %pos_result = select i1 %is_neg, i128 %m, i128 0
  %8 = select i1 %is_neg, i32* inttoptr (i64 add (i64 ptrtoint ([3 x i32]* @__sancov_gen_.11 to i64), i64 4) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([3 x i32]* @__sancov_gen_.11 to i64), i64 8) to i32*)
  %9 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %10 = load i32, i32* %8, align 4, !nosanitize !0
  %11 = getelementptr i8, i8* %9, i32 %10
  %12 = load i8, i8* %11, align 1, !nosanitize !0
  %13 = add i8 %12, 1
  %14 = icmp eq i8 %13, 0
  %15 = zext i1 %14 to i8
  %16 = add i8 %13, %15
  store i8 %16, i8* %11, align 1, !nosanitize !0
  %final_result = add i128 %pos_result, %result
  ret i128 %final_result
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind readnone willreturn
define i128 @mod_sub(i128 %a, i128 %b, i128 %m) local_unnamed_addr #1 comdat {
entry:
  %0 = load i32, i32* getelementptr inbounds ([3 x i32], [3 x i32]* @__sancov_gen_.12, i32 0, i32 0), align 4, !nosanitize !0
  %1 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %2 = getelementptr i8, i8* %1, i32 %0
  %3 = load i8, i8* %2, align 1, !nosanitize !0
  %4 = add i8 %3, 1
  %5 = icmp eq i8 %4, 0
  %6 = zext i1 %5 to i8
  %7 = add i8 %4, %6
  store i8 %7, i8* %2, align 1, !nosanitize !0
  %sub = sub i128 %a, %b
  %result = srem i128 %sub, %m
  %is_neg = icmp slt i128 %result, 0
  %pos_result = select i1 %is_neg, i128 %m, i128 0
  %8 = select i1 %is_neg, i32* inttoptr (i64 add (i64 ptrtoint ([3 x i32]* @__sancov_gen_.12 to i64), i64 4) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([3 x i32]* @__sancov_gen_.12 to i64), i64 8) to i32*)
  %9 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %10 = load i32, i32* %8, align 4, !nosanitize !0
  %11 = getelementptr i8, i8* %9, i32 %10
  %12 = load i8, i8* %11, align 1, !nosanitize !0
  %13 = add i8 %12, 1
  %14 = icmp eq i8 %13, 0
  %15 = zext i1 %14 to i8
  %16 = add i8 %13, %15
  store i8 %16, i8* %11, align 1, !nosanitize !0
  %final_result = add i128 %pos_result, %result
  ret i128 %final_result
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind readnone willreturn
define i128 @mod_mul(i128 %a, i128 %b, i128 %m) local_unnamed_addr #1 comdat {
entry:
  %0 = load i32, i32* getelementptr inbounds ([3 x i32], [3 x i32]* @__sancov_gen_.13, i32 0, i32 0), align 4, !nosanitize !0
  %1 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %2 = getelementptr i8, i8* %1, i32 %0
  %3 = load i8, i8* %2, align 1, !nosanitize !0
  %4 = add i8 %3, 1
  %5 = icmp eq i8 %4, 0
  %6 = zext i1 %5 to i8
  %7 = add i8 %4, %6
  store i8 %7, i8* %2, align 1, !nosanitize !0
  %prod = mul i128 %b, %a
  %result = srem i128 %prod, %m
  %is_neg = icmp slt i128 %result, 0
  %pos_result = select i1 %is_neg, i128 %m, i128 0
  %8 = select i1 %is_neg, i32* inttoptr (i64 add (i64 ptrtoint ([3 x i32]* @__sancov_gen_.13 to i64), i64 4) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([3 x i32]* @__sancov_gen_.13 to i64), i64 8) to i32*)
  %9 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %10 = load i32, i32* %8, align 4, !nosanitize !0
  %11 = getelementptr i8, i8* %9, i32 %10
  %12 = load i8, i8* %11, align 1, !nosanitize !0
  %13 = add i8 %12, 1
  %14 = icmp eq i8 %13, 0
  %15 = zext i1 %14 to i8
  %16 = add i8 %13, %15
  store i8 %16, i8* %11, align 1, !nosanitize !0
  %final_result = add i128 %pos_result, %result
  ret i128 %final_result
}

; Function Attrs: nounwind
define i128 @mod_inverse(i128 %input, i128 %modulus) local_unnamed_addr #6 comdat {
entry:
  %0 = load i32, i32* getelementptr inbounds ([10 x i32], [10 x i32]* @__sancov_gen_.14, i32 0, i32 0), align 4, !nosanitize !0
  %1 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %2 = getelementptr i8, i8* %1, i32 %0
  %3 = load i8, i8* %2, align 1, !nosanitize !0
  %4 = add i8 %3, 1
  %5 = icmp eq i8 %4, 0
  %6 = zext i1 %5 to i8
  %7 = add i8 %4, %6
  store i8 %7, i8* %2, align 1, !nosanitize !0
  %8 = tail call i128 @llvm.abs.i128(i128 %modulus, i1 false)
  %new_r_neg = icmp slt i128 %input, 0
  %new_r_add = select i1 %new_r_neg, i128 %modulus, i128 0
  %9 = select i1 %new_r_neg, i32* inttoptr (i64 add (i64 ptrtoint ([10 x i32]* @__sancov_gen_.14 to i64), i64 24) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([10 x i32]* @__sancov_gen_.14 to i64), i64 28) to i32*)
  %10 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %11 = load i32, i32* %9, align 4, !nosanitize !0
  %12 = getelementptr i8, i8* %10, i32 %11
  %13 = load i8, i8* %12, align 1, !nosanitize !0
  %14 = add i8 %13, 1
  %15 = icmp eq i8 %14, 0
  %16 = zext i1 %15 to i8
  %17 = add i8 %14, %16
  store i8 %17, i8* %12, align 1, !nosanitize !0
  %new_r_abs = add i128 %new_r_add, %input
  %is_zero3 = icmp eq i128 %new_r_abs, 0
  br i1 %is_zero3, label %entry.end_crit_edge, label %entry.continue_crit_edge

entry.continue_crit_edge:                         ; preds = %entry
  br label %continue

entry.end_crit_edge:                              ; preds = %entry
  %18 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([10 x i32]* @__sancov_gen_.14 to i64), i64 4) to i32*), align 4, !nosanitize !0
  %19 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %20 = getelementptr i8, i8* %19, i32 %18
  %21 = load i8, i8* %20, align 1, !nosanitize !0
  %22 = add i8 %21, 1
  %23 = icmp eq i8 %22, 0
  %24 = zext i1 %23 to i8
  %25 = add i8 %22, %24
  store i8 %25, i8* %20, align 1, !nosanitize !0
  br label %end

continue:                                         ; preds = %continue.continue_crit_edge, %entry.continue_crit_edge
  %new_r.07 = phi i128 [ %new_r_updated, %continue.continue_crit_edge ], [ %new_r_abs, %entry.continue_crit_edge ]
  %r.06 = phi i128 [ %new_r.07, %continue.continue_crit_edge ], [ %8, %entry.continue_crit_edge ]
  %new_t.05 = phi i128 [ %new_t_updated, %continue.continue_crit_edge ], [ 1, %entry.continue_crit_edge ]
  %t.04 = phi i128 [ %new_t.05, %continue.continue_crit_edge ], [ 0, %entry.continue_crit_edge ]
  %quotient = sdiv i128 %r.06, %new_r.07
  %temp_t = mul i128 %quotient, %new_t.05
  %new_t_updated = sub i128 %t.04, %temp_t
  %temp_r = mul i128 %quotient, %new_r.07
  %new_r_updated = sub i128 %r.06, %temp_r
  %is_zero = icmp eq i128 %new_r_updated, 0
  br i1 %is_zero, label %continue.end_crit_edge, label %continue.continue_crit_edge

continue.continue_crit_edge:                      ; preds = %continue
  %26 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([10 x i32]* @__sancov_gen_.14 to i64), i64 8) to i32*), align 4, !nosanitize !0
  %27 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %28 = getelementptr i8, i8* %27, i32 %26
  %29 = load i8, i8* %28, align 1, !nosanitize !0
  %30 = add i8 %29, 1
  %31 = icmp eq i8 %30, 0
  %32 = zext i1 %31 to i8
  %33 = add i8 %30, %32
  store i8 %33, i8* %28, align 1, !nosanitize !0
  br label %continue

continue.end_crit_edge:                           ; preds = %continue
  %34 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([10 x i32]* @__sancov_gen_.14 to i64), i64 12) to i32*), align 4, !nosanitize !0
  %35 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %36 = getelementptr i8, i8* %35, i32 %34
  %37 = load i8, i8* %36, align 1, !nosanitize !0
  %38 = add i8 %37, 1
  %39 = icmp eq i8 %38, 0
  %40 = zext i1 %39 to i8
  %41 = add i8 %38, %40
  store i8 %41, i8* %36, align 1, !nosanitize !0
  br label %end

end:                                              ; preds = %continue.end_crit_edge, %entry.end_crit_edge
  %t.0.lcssa = phi i128 [ 0, %entry.end_crit_edge ], [ %new_t.05, %continue.end_crit_edge ]
  %r.0.lcssa = phi i128 [ %8, %entry.end_crit_edge ], [ %new_r.07, %continue.end_crit_edge ]
  %inverse_exists = icmp eq i128 %r.0.lcssa, 1
  br i1 %inverse_exists, label %compute_result, label %error

error:                                            ; preds = %end
  %42 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([10 x i32]* @__sancov_gen_.14 to i64), i64 16) to i32*), align 4, !nosanitize !0
  %43 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %44 = getelementptr i8, i8* %43, i32 %42
  %45 = load i8, i8* %44, align 1, !nosanitize !0
  %46 = add i8 %45, 1
  %47 = icmp eq i8 %46, 0
  %48 = zext i1 %47 to i8
  %49 = add i8 %46, %48
  store i8 %49, i8* %44, align 1, !nosanitize !0
  tail call void @llvm.trap()
  unreachable

compute_result:                                   ; preds = %end
  %50 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([10 x i32]* @__sancov_gen_.14 to i64), i64 20) to i32*), align 4, !nosanitize !0
  %51 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %52 = getelementptr i8, i8* %51, i32 %50
  %53 = load i8, i8* %52, align 1, !nosanitize !0
  %54 = add i8 %53, 1
  %55 = icmp eq i8 %54, 0
  %56 = zext i1 %55 to i8
  %57 = add i8 %54, %56
  store i8 %57, i8* %52, align 1, !nosanitize !0
  %result_neg = icmp slt i128 %t.0.lcssa, 0
  %result_add = select i1 %result_neg, i128 %modulus, i128 0
  %58 = select i1 %result_neg, i32* inttoptr (i64 add (i64 ptrtoint ([10 x i32]* @__sancov_gen_.14 to i64), i64 32) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([10 x i32]* @__sancov_gen_.14 to i64), i64 36) to i32*)
  %59 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %60 = load i32, i32* %58, align 4, !nosanitize !0
  %61 = getelementptr i8, i8* %59, i32 %60
  %62 = load i8, i8* %61, align 1, !nosanitize !0
  %63 = add i8 %62, 1
  %64 = icmp eq i8 %63, 0
  %65 = zext i1 %64 to i8
  %66 = add i8 %63, %65
  store i8 %66, i8* %61, align 1, !nosanitize !0
  %result_pos = add i128 %result_add, %t.0.lcssa
  %final_result = srem i128 %result_pos, %modulus
  ret i128 %final_result
}

; Function Attrs: nounwind
define i128 @mod_div(i128 %a, i128 %b, i128 %m) local_unnamed_addr #6 comdat {
entry:
  %0 = load i32, i32* getelementptr inbounds ([13 x i32], [13 x i32]* @__sancov_gen_.15, i32 0, i32 0), align 4, !nosanitize !0
  %1 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %2 = getelementptr i8, i8* %1, i32 %0
  %3 = load i8, i8* %2, align 1, !nosanitize !0
  %4 = add i8 %3, 1
  %5 = icmp eq i8 %4, 0
  %6 = zext i1 %5 to i8
  %7 = add i8 %4, %6
  store i8 %7, i8* %2, align 1, !nosanitize !0
  %b_is_zero = icmp eq i128 %b, 0
  br i1 %b_is_zero, label %entry.common.ret_crit_edge, label %compute_inverse

entry.common.ret_crit_edge:                       ; preds = %entry
  %8 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([13 x i32]* @__sancov_gen_.15 to i64), i64 4) to i32*), align 4, !nosanitize !0
  %9 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %10 = getelementptr i8, i8* %9, i32 %8
  %11 = load i8, i8* %10, align 1, !nosanitize !0
  %12 = add i8 %11, 1
  %13 = icmp eq i8 %12, 0
  %14 = zext i1 %13 to i8
  %15 = add i8 %12, %14
  store i8 %15, i8* %10, align 1, !nosanitize !0
  br label %common.ret

common.ret:                                       ; preds = %entry.common.ret_crit_edge, %mod_inverse.exit
  %common.ret.op = phi i128 [ %final_result.i1, %mod_inverse.exit ], [ 0, %entry.common.ret_crit_edge ]
  ret i128 %common.ret.op

compute_inverse:                                  ; preds = %entry
  %16 = tail call i128 @llvm.abs.i128(i128 %m, i1 false) #6
  %new_r_neg.i = icmp slt i128 %b, 0
  %new_r_add.i = select i1 %new_r_neg.i, i128 %m, i128 0
  %17 = select i1 %new_r_neg.i, i32* inttoptr (i64 add (i64 ptrtoint ([13 x i32]* @__sancov_gen_.15 to i64), i64 28) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([13 x i32]* @__sancov_gen_.15 to i64), i64 32) to i32*)
  %18 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %19 = load i32, i32* %17, align 4, !nosanitize !0
  %20 = getelementptr i8, i8* %18, i32 %19
  %21 = load i8, i8* %20, align 1, !nosanitize !0
  %22 = add i8 %21, 1
  %23 = icmp eq i8 %22, 0
  %24 = zext i1 %23 to i8
  %25 = add i8 %22, %24
  store i8 %25, i8* %20, align 1, !nosanitize !0
  %new_r_abs.i = add i128 %new_r_add.i, %b
  %is_zero3.i = icmp eq i128 %new_r_abs.i, 0
  br i1 %is_zero3.i, label %compute_inverse.end.i_crit_edge, label %compute_inverse.continue.i_crit_edge

compute_inverse.continue.i_crit_edge:             ; preds = %compute_inverse
  br label %continue.i

compute_inverse.end.i_crit_edge:                  ; preds = %compute_inverse
  %26 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([13 x i32]* @__sancov_gen_.15 to i64), i64 8) to i32*), align 4, !nosanitize !0
  %27 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %28 = getelementptr i8, i8* %27, i32 %26
  %29 = load i8, i8* %28, align 1, !nosanitize !0
  %30 = add i8 %29, 1
  %31 = icmp eq i8 %30, 0
  %32 = zext i1 %31 to i8
  %33 = add i8 %30, %32
  store i8 %33, i8* %28, align 1, !nosanitize !0
  br label %end.i

continue.i:                                       ; preds = %continue.i.continue.i_crit_edge, %compute_inverse.continue.i_crit_edge
  %new_r.07.i = phi i128 [ %new_r_updated.i, %continue.i.continue.i_crit_edge ], [ %new_r_abs.i, %compute_inverse.continue.i_crit_edge ]
  %r.06.i = phi i128 [ %new_r.07.i, %continue.i.continue.i_crit_edge ], [ %16, %compute_inverse.continue.i_crit_edge ]
  %new_t.05.i = phi i128 [ %new_t_updated.i, %continue.i.continue.i_crit_edge ], [ 1, %compute_inverse.continue.i_crit_edge ]
  %t.04.i = phi i128 [ %new_t.05.i, %continue.i.continue.i_crit_edge ], [ 0, %compute_inverse.continue.i_crit_edge ]
  %quotient.i = sdiv i128 %r.06.i, %new_r.07.i
  %temp_t.i = mul i128 %quotient.i, %new_t.05.i
  %new_t_updated.i = sub i128 %t.04.i, %temp_t.i
  %temp_r.i = mul i128 %quotient.i, %new_r.07.i
  %new_r_updated.i = sub i128 %r.06.i, %temp_r.i
  %is_zero.i = icmp eq i128 %new_r_updated.i, 0
  br i1 %is_zero.i, label %continue.i.end.i_crit_edge, label %continue.i.continue.i_crit_edge

continue.i.continue.i_crit_edge:                  ; preds = %continue.i
  %34 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([13 x i32]* @__sancov_gen_.15 to i64), i64 12) to i32*), align 4, !nosanitize !0
  %35 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %36 = getelementptr i8, i8* %35, i32 %34
  %37 = load i8, i8* %36, align 1, !nosanitize !0
  %38 = add i8 %37, 1
  %39 = icmp eq i8 %38, 0
  %40 = zext i1 %39 to i8
  %41 = add i8 %38, %40
  store i8 %41, i8* %36, align 1, !nosanitize !0
  br label %continue.i

continue.i.end.i_crit_edge:                       ; preds = %continue.i
  %42 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([13 x i32]* @__sancov_gen_.15 to i64), i64 16) to i32*), align 4, !nosanitize !0
  %43 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %44 = getelementptr i8, i8* %43, i32 %42
  %45 = load i8, i8* %44, align 1, !nosanitize !0
  %46 = add i8 %45, 1
  %47 = icmp eq i8 %46, 0
  %48 = zext i1 %47 to i8
  %49 = add i8 %46, %48
  store i8 %49, i8* %44, align 1, !nosanitize !0
  br label %end.i

end.i:                                            ; preds = %continue.i.end.i_crit_edge, %compute_inverse.end.i_crit_edge
  %t.0.lcssa.i = phi i128 [ 0, %compute_inverse.end.i_crit_edge ], [ %new_t.05.i, %continue.i.end.i_crit_edge ]
  %r.0.lcssa.i = phi i128 [ %16, %compute_inverse.end.i_crit_edge ], [ %new_r.07.i, %continue.i.end.i_crit_edge ]
  %inverse_exists.i = icmp eq i128 %r.0.lcssa.i, 1
  br i1 %inverse_exists.i, label %mod_inverse.exit, label %error.i

error.i:                                          ; preds = %end.i
  %50 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([13 x i32]* @__sancov_gen_.15 to i64), i64 20) to i32*), align 4, !nosanitize !0
  %51 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %52 = getelementptr i8, i8* %51, i32 %50
  %53 = load i8, i8* %52, align 1, !nosanitize !0
  %54 = add i8 %53, 1
  %55 = icmp eq i8 %54, 0
  %56 = zext i1 %55 to i8
  %57 = add i8 %54, %56
  store i8 %57, i8* %52, align 1, !nosanitize !0
  tail call void @llvm.trap() #6
  unreachable

mod_inverse.exit:                                 ; preds = %end.i
  %58 = load i32, i32* inttoptr (i64 add (i64 ptrtoint ([13 x i32]* @__sancov_gen_.15 to i64), i64 24) to i32*), align 4, !nosanitize !0
  %59 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %60 = getelementptr i8, i8* %59, i32 %58
  %61 = load i8, i8* %60, align 1, !nosanitize !0
  %62 = add i8 %61, 1
  %63 = icmp eq i8 %62, 0
  %64 = zext i1 %63 to i8
  %65 = add i8 %62, %64
  store i8 %65, i8* %60, align 1, !nosanitize !0
  %result_neg.i = icmp slt i128 %t.0.lcssa.i, 0
  %result_add.i = select i1 %result_neg.i, i128 %m, i128 0
  %66 = select i1 %result_neg.i, i32* inttoptr (i64 add (i64 ptrtoint ([13 x i32]* @__sancov_gen_.15 to i64), i64 36) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([13 x i32]* @__sancov_gen_.15 to i64), i64 40) to i32*)
  %67 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %68 = load i32, i32* %66, align 4, !nosanitize !0
  %69 = getelementptr i8, i8* %67, i32 %68
  %70 = load i8, i8* %69, align 1, !nosanitize !0
  %71 = add i8 %70, 1
  %72 = icmp eq i8 %71, 0
  %73 = zext i1 %72 to i8
  %74 = add i8 %71, %73
  store i8 %74, i8* %69, align 1, !nosanitize !0
  %result_pos.i = add i128 %result_add.i, %t.0.lcssa.i
  %final_result.i = srem i128 %result_pos.i, %m
  %prod.i = mul i128 %final_result.i, %a
  %result.i = srem i128 %prod.i, %m
  %is_neg.i = icmp slt i128 %result.i, 0
  %pos_result.i = select i1 %is_neg.i, i128 %m, i128 0
  %75 = select i1 %is_neg.i, i32* inttoptr (i64 add (i64 ptrtoint ([13 x i32]* @__sancov_gen_.15 to i64), i64 44) to i32*), i32* inttoptr (i64 add (i64 ptrtoint ([13 x i32]* @__sancov_gen_.15 to i64), i64 48) to i32*)
  %76 = load i8*, i8** @__afl_area_ptr, align 8, !nosanitize !0
  %77 = load i32, i32* %75, align 4, !nosanitize !0
  %78 = getelementptr i8, i8* %76, i32 %77
  %79 = load i8, i8* %78, align 1, !nosanitize !0
  %80 = add i8 %79, 1
  %81 = icmp eq i8 %80, 0
  %82 = zext i1 %81 to i8
  %83 = add i8 %80, %82
  store i8 %83, i8* %78, align 1, !nosanitize !0
  %final_result.i1 = add i128 %pos_result.i, %result.i
  br label %common.ret
}

; Function Attrs: nofree nounwind
declare noundef i32 @puts(i8* nocapture noundef readonly) local_unnamed_addr #7

; Function Attrs: nofree nosync nounwind readnone speculatable willreturn
declare i128 @llvm.abs.i128(i128, i1 immarg) #9

declare void @__sanitizer_cov_trace_pc_guard_init(i32**, i32**)

; Function Attrs: nounwind
define internal void @sancov.module_ctor_trace_pc_guard() #6 comdat {
  call void @__sanitizer_cov_trace_pc_guard_init(i32** @__start___sancov_guards, i32** @__stop___sancov_guards)
  ret void
}

attributes #0 = { mustprogress nofree norecurse nosync nounwind willreturn writeonly }
attributes #1 = { mustprogress nofree norecurse nosync nounwind readnone willreturn }
attributes #2 = { mustprogress nofree nosync nounwind readnone willreturn }
attributes #3 = { mustprogress nofree nosync nounwind readnone speculatable willreturn }
attributes #4 = { mustprogress nofree nounwind willreturn }
attributes #5 = { inaccessiblememonly mustprogress nofree nounwind willreturn }
attributes #6 = { nounwind }
attributes #7 = { nofree nounwind }
attributes #8 = { cold noreturn nounwind }
attributes #9 = { nofree nosync nounwind readnone speculatable willreturn }

!0 = !{}
