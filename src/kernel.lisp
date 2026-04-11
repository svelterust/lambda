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

(cffi:defcfun ("lambda_run"           %run)           :void)
(cffi:defcfun ("lambda_buf_ptr"       %buf-ptr)       :pointer)
(cffi:defcfun ("lambda_buf_set_count" %buf-set-count) :void (n :uint32))

(cffi:defcstruct draw-cmd
  (x     :float)
  (y     :float)
  (w     :float)
  (h     :float)
  (color :uint32))

;; Functions to write into shared memory
(defparameter *buf* (%buf-ptr))
(defvar *idx* 0)

(defun clear ()
  "Reset the draw command index to zero."
  (setf *idx* 0))

(defun rect (rx ry rw rh rc)
  "Write a rectangle draw command to the shared buffer."
  (declare (type single-float rx ry rw rh)
           (type (unsigned-byte 32) rc))
  (assert (< *idx* 1024))
  (let ((ptr (cffi:inc-pointer *buf* (* *idx* (cffi:foreign-type-size '(:struct draw-cmd))))))
    (cffi:with-foreign-slots ((x y w h color) ptr (:struct draw-cmd))
      (setf x rx y ry w rw h rh color rc)))
  (incf *idx*))

(defun flush ()
  "Publish the current draw commands to the renderer."
  (%buf-set-count *idx*)
  *idx*)

(defun run ()
  "Start the kernel event loop on a background thread."
  (bt:make-thread #'%run :name "lambda"))
