(in-package :lambda)

;; Load libkernel
(let ((lib-dir (merge-pathnames
                #p"kernel/target/debug/"
                (asdf:system-source-directory "lambda"))))
  (pushnew lib-dir cffi:*foreign-library-directories* :test #'equal))

(cffi:define-foreign-library libkernel
  (:unix (:default "libkernel")))

(when (cffi:foreign-library-loaded-p 'libkernel)
  (cffi:close-foreign-library 'libkernel))
(cffi:use-foreign-library libkernel)

;; FFI
(cffi:defcfun ("lambda_run"           %run)           :void)
(cffi:defcfun ("lambda_buf_ptr"       %buf-ptr)       :pointer)
(cffi:defcfun ("lambda_buf_set_count" %buf-set-count) :void (n :uint32))

(cffi:defcstruct draw-cmd
  (x     :float)
  (y     :float)
  (w     :float)
  (h     :float)
  (color :uint32))

;; Shared buffer pointer (cached once at load time)
(defvar *buf* (%buf-ptr))
(defvar *idx* 0)

(defun clear ()
  (setf *idx* 0))

(defun rect (rx ry rw rh rc)
  (let ((ptr (cffi:inc-pointer *buf* (* *idx* (cffi:foreign-type-size '(:struct draw-cmd))))))
    (cffi:with-foreign-slots ((x y w h color) ptr (:struct draw-cmd))
      (setf x (float rx) y (float ry) w (float rw) h (float rh) color rc)))
  (incf *idx*))

(defun flush ()
  (%buf-set-count *idx*))

(defun run ()
  (bt:make-thread #'%run :name "lambda"))
