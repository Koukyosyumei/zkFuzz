declare void @llvm.trap()

; Function to perform modular addition (a + b mod m)
define i128 @mod_add(i128 %a, i128 %b, i128 %m) {
entry:
  %add = add i128 %a, %b
  %result = srem i128 %add, %m
  %is_neg = icmp slt i128 %result, 0
  %pos_result = add i128 %result, %m
  %final_result = select i1 %is_neg, i128 %pos_result, i128 %result
  ret i128 %final_result
}

; Function to perform modular subtraction (a - b mod m)
define i128 @mod_sub(i128 %a, i128 %b, i128 %m) {
entry:
  %sub = sub i128 %a, %b
  %result = srem i128 %sub, %m
  %is_neg = icmp slt i128 %result, 0
  %pos_result = add i128 %result, %m
  %final_result = select i1 %is_neg, i128 %pos_result, i128 %result
  ret i128 %final_result
}

; Function to perform modular multiplication (a * b mod m)
define i128 @mod_mul(i128 %a, i128 %b, i128 %m) {
entry:
  %prod = mul i128 %a, %b
  %result = srem i128 %prod, %m
  %is_neg = icmp slt i128 %result, 0
  %pos_result = add i128 %result, %m
  %final_result = select i1 %is_neg, i128 %pos_result, i128 %result
  ret i128 %final_result
}

; Function to perform modular inversion (1 / a mod m)
define i128 @mod_inverse(i128 %input, i128 %modulus) {
entry:
  %t = alloca i128
  %new_t = alloca i128
  %r = alloca i128
  %new_r = alloca i128
  
  store i128 0, i128* %t
  store i128 1, i128* %new_t
  store i128 %modulus, i128* %r
  store i128 %input, i128* %new_r
  
  ; Ensure r and new_r are non-negative
  %r_neg = icmp slt i128 %modulus, 0
  %r_sub = sub i128 0, %modulus
  %r_abs = select i1 %r_neg, i128 %r_sub, i128 %modulus
  store i128 %r_abs, i128* %r
  
  %new_r_neg = icmp slt i128 %input, 0
  %new_r_add = add i128 %input, %modulus
  %new_r_abs = select i1 %new_r_neg, i128 %new_r_add, i128 %input
  store i128 %new_r_abs, i128* %new_r
  
  br label %loop

loop:
  %new_r_val = load i128, i128* %new_r
  %is_zero = icmp eq i128 %new_r_val, 0
  br i1 %is_zero, label %end, label %continue

continue:
  %r_val = load i128, i128* %r
  %quotient = sdiv i128 %r_val, %new_r_val
  
  ; Update t and new_t
  %t_val = load i128, i128* %t
  %new_t_val = load i128, i128* %new_t
  %temp_t = mul i128 %quotient, %new_t_val
  %new_t_updated = sub i128 %t_val, %temp_t
  store i128 %new_t_val, i128* %t
  store i128 %new_t_updated, i128* %new_t
  
  ; Update r and new_r
  %temp_r = mul i128 %quotient, %new_r_val
  %new_r_updated = sub i128 %r_val, %temp_r
  store i128 %new_r_val, i128* %r
  store i128 %new_r_updated, i128* %new_r
  
  br label %loop

end:
  %final_r = load i128, i128* %r
  %inverse_exists = icmp eq i128 %final_r, 1
  br i1 %inverse_exists, label %compute_result, label %error

error:
  call void @llvm.trap()
  unreachable

compute_result:
  %result = load i128, i128* %t
  %result_neg = icmp slt i128 %result, 0
  %result_add = add i128 %result, %modulus
  %result_pos = select i1 %result_neg, i128 %result_add, i128 %result
  %final_result = srem i128 %result_pos, %modulus
  ret i128 %final_result
}

declare i32 @printf(i8*, ...)

@.str = private constant [5 x i8] c"%ld\0A\00"

; Function to perform modular division (a / b mod m)
define i128 @mod_div(i128 %a, i128 %b, i128 %m) {
entry:
  ; Check if b is zero
  %b_is_zero = icmp eq i128 %b, 0
  br i1 %b_is_zero, label %return_zero, label %compute_inverse

return_zero:
  ret i128 0

compute_inverse:
  ; First, compute the modular inverse of b
  %b_inv = call i128 @mod_inverse(i128 %b, i128 %m)
  
  ; Now multiply a with b_inv modulo m
  %result = call i128 @mod_mul(i128 %a, i128 %b_inv, i128 %m)
  
  ret i128 %result
}