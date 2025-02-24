(set-logic AUFLIA)
(declare-const p Int)
(assert (= p 21888242871839275222246405745257275088548364400416034343698204186575808495617))

(define-fun fadd ((x Int) (y Int)) Int (mod_total (+ x y) p))
(define-fun fsub ((x Int) (y Int)) Int (mod_total (- x y) p))
(define-fun fmul ((x Int) (y Int)) Int (mod_total (* x y) p))

; --- Declare variables ---
(declare-const var_15726138482267553814 Int)
(declare-const var_17358476748517776603 Int)
(declare-const var_7318935578537365113 Int)

; --- Assertions ---
; (assert (= var_17358476748517776603 (fdiv var_15726138482267553814 var_7318935578537365113)))
(assert (= (fmul var_17358476748517776603 var_7318935578537365113) var_15726138482267553814))
(assert (= var_15726138482267553814 0))
(assert (= var_7318935578537365113 0))

(check-sat)
(get-model)
