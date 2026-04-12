(in-package :lambda)

;;; FFI
(cffi:defcfun ("lambda_image_create"       %image-create)     :uint32  (ptr :pointer) (len :uint32))
(cffi:defcfun ("lambda_image_destroy"      image-destroy)     :void    (id :uint32))
(cffi:defcfun ("lambda_image_position"     image-position)    :void    (id :uint32) (x :number) (y :number))
(cffi:defcfun ("lambda_image_size"         image-size)        :void    (id :uint32) (w :number) (h :number))
(cffi:defcfun ("lambda_image_width"        image-width)       :uint32  (id :uint32))
(cffi:defcfun ("lambda_image_height"       image-height)      :uint32  (id :uint32))
(cffi:defcfun ("lambda_image_aspect_ratio" image-aspect-ratio) :float  (id :uint32))

(defun make-image (path)
  (let ((bytes (sb-ext:string-to-octets (namestring path) :external-format :utf-8)))
    (cffi:with-pointer-to-vector-data (ptr bytes)
      (%image-create ptr (length bytes)))))
