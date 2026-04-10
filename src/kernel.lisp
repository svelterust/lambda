(in-package :lambda)

;; Load kernel from Rust
(let ((lib-dir (merge-pathnames
                #p"kernel/target/debug/"
                (asdf:system-source-directory "lambda"))))
  (pushnew lib-dir cffi:*foreign-library-directories* :test #'equal))

(cffi:define-foreign-library libkernel
  (:unix (:default "libkernel")))

(cffi:use-foreign-library libkernel)

(cffi:defcfun ("lambda_run" %run) :void)
