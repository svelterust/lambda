(in-package :lambda)

(let ((lib-dir (merge-pathnames
                #p"kernel/target/debug/"
                (asdf:system-source-directory "lambda"))))
  (pushnew lib-dir cffi:*foreign-library-directories* :test #'equal))

(cffi:define-foreign-library libkernel
  (:unix (:default "libkernel")))

(when (cffi:foreign-library-loaded-p 'libkernel)
  (cffi:close-foreign-library 'libkernel))
(cffi:use-foreign-library libkernel)

;; Run
(cffi:defcfun ("lambda_run" %run) :void)

(unless (find "lambda" (bt:all-threads) :key #'bt:thread-name :test #'string=)
  (bt:make-thread #'%run :name "lambda"))

