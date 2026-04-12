(defpackage :lambda
  (:use :cl)
  (:export :run))

(in-package :lambda)

(let ((lib-dir (merge-pathnames
                #p"lib/target/debug/"
                (asdf:system-source-directory "lambda"))))
  (pushnew lib-dir cffi:*foreign-library-directories* :test #'equal))

(cffi:define-foreign-library liblambda
  (:unix (:default "liblambda")))
(cffi:use-foreign-library liblambda)

;; Run
(cffi:defcfun ("lambda_run" %run) :void)
(bt:make-thread #'%run :name "lambda")

