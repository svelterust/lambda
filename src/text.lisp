(in-package :lambda)

;;; FFI
(cffi:defcfun ("lambda_text_create"   %text-create)   :uint32
  (font-size :float) (line-height :float))

(cffi:defcfun ("lambda_text_destroy"  %text-destroy)  :void
  (id :uint32))

(cffi:defcfun ("lambda_text_set"      %text-set)      :void
  (id :uint32) (ptr :pointer) (len :uint32))

(cffi:defcfun ("lambda_text_position" %text-position) :void
  (id :uint32) (x :float) (y :float))

(cffi:defcfun ("lambda_text_bounds"   %text-bounds)   :void
  (id :uint32) (left :int32) (top :int32) (right :int32) (bottom :int32))

(cffi:defcfun ("lambda_text_color"    %text-color)    :void
  (id :uint32) (rgba :uint32))

(cffi:defcfun ("lambda_text_metrics"  %text-metrics)  :void
  (id :uint32) (font-size :float) (line-height :float))

;; Text
(defun make-text (&key (size 16.0) (line-height (* size 1.4)))
  "Create a new text area. Returns an integer ID."
  (declare (type single-float size line-height))
  (%text-create size line-height))

(defun text-destroy (id)
  "Destroy a text area."
  (declare (type (unsigned-byte 32) id))
  (%text-destroy id))

(defun text-set (id string)
  "Set the text content of a text area."
  (declare (type (unsigned-byte 32) id)
           (type string string))
  (let ((octets (sb-ext:string-to-octets string :external-format :utf-8)))
    (sb-sys:with-pinned-objects (octets)
      (%text-set id (sb-sys:vector-sap octets) (length octets)))))

(defun text-position (id x y)
  "Set the screen position of a text area."
  (declare (type (unsigned-byte 32) id)
           (type single-float x y))
  (%text-position id x y))

(defun text-bounds (id left top right bottom)
  "Set the clip bounds of a text area."
  (declare (type (unsigned-byte 32) id)
           (type (signed-byte 32) left top right bottom))
  (%text-bounds id left top right bottom))

(defun text-color (id rgba)
  "Set the default text color as #xRRGGBBAA."
  (declare (type (unsigned-byte 32) id rgba))
  (%text-color id rgba))

(defun text-metrics (id font-size line-height)
  "Set font size and line height."
  (declare (type (unsigned-byte 32) id)
           (type single-float font-size line-height))
  (%text-metrics id font-size line-height))
