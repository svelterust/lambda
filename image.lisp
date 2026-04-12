(in-package :lambda)

;;; FFI
(cffi:defcfun ("lambda_image_create"   %image-create)   :uint32
  (ptr :pointer) (len :uint32))

(cffi:defcfun ("lambda_image_destroy"  %image-destroy)  :void
  (id :uint32))

(cffi:defcfun ("lambda_image_position" %image-position) :void
  (id :uint32) (x :float) (y :float))

(cffi:defcfun ("lambda_image_size"     %image-size)     :void
  (id :uint32) (w :float) (h :float))

(cffi:defcfun ("lambda_image_width"   %image-width)   :uint32
  (id :uint32))

(cffi:defcfun ("lambda_image_height"  %image-height)  :uint32
  (id :uint32))

(cffi:defcfun ("lambda_image_aspect_ratio" %image-aspect-ratio) :float
  (id :uint32))

;;; Image
(defun make-image (path)
  "Load an image from PATH. Returns an integer ID, or 0 on failure."
  (declare (type (or string pathname) path))
  (let ((bytes (sb-ext:string-to-octets (namestring path) :external-format :utf-8)))
    (cffi:with-pointer-to-vector-data (ptr bytes)
      (%image-create ptr (length bytes)))))

(defun image-destroy (id)
  "Destroy an image."
  (declare (type (unsigned-byte 32) id))
  (%image-destroy id))

(defun image-position (id x y)
  "Set the screen position of an image."
  (declare (type (unsigned-byte 32) id)
           (type single-float x y))
  (%image-position id x y))

(defun image-size (id w h)
  "Set the display size of an image."
  (declare (type (unsigned-byte 32) id)
           (type single-float w h))
  (%image-size id w h))

(defun image-width (id)
  "Get the natural width of an image."
  (declare (type (unsigned-byte 32) id))
  (%image-width id))

(defun image-height (id)
  "Get the natural height of an image."
  (declare (type (unsigned-byte 32) id))
  (%image-height id))

(defun image-aspect-ratio (id)
  "Get the aspect ratio (width/height) of an image."
  (declare (type (unsigned-byte 32) id))
  (%image-aspect-ratio id))
