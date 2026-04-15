(defpackage :lambda
  (:use :cl)
  (:export :run))

(in-package :lambda)

;;; Custom CFFI type for number -> float
(cffi:define-foreign-type number-as-float ()
  ()
  (:actual-type :float)
  (:simple-parser :number))

(defmethod cffi:translate-to-foreign (value (type number-as-float))
  (float value))

;;; Library
(let ((lib-dir (merge-pathnames
                #p"lib/target/debug/"
                (asdf:system-source-directory "lambda"))))
  (pushnew lib-dir cffi:*foreign-library-directories* :test #'equal))

(cffi:define-foreign-library liblambda
  (:unix (:default "liblambda")))
(cffi:use-foreign-library liblambda)

;;; Thin wrapper: auto-prepend "lambda_" to the C name
(defmacro defcfun (c-name lisp-name return-type &rest args)
  `(cffi:defcfun (,(concatenate 'string "lambda_" c-name) ,lisp-name) ,return-type ,@args))

;; Run
(defcfun "run" run :void)
(bt:make-thread #'run :name "lambda")

