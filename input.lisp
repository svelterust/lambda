(in-package :lambda)

;;; FFI
(cffi:defcstruct input-event
  (event-type :uint8)
  (modifiers  :uint8)
  (code       :uint16)
  (x          :float)
  (y          :float))

(defcfun "input_buf_ptr"        %input-buf-ptr        :pointer)
(defcfun "input_write_index"    %input-write-index    :uint32)
(defcfun "input_set_read_index" %input-set-read-index :void (n :uint32))

;;; Lookup tables (numeric -> keyword)
(defvar *event-types*
  #(:unknown :key-down :key-up :mouse-move :mouse-down :mouse-up :scroll))

(defvar *mouse-buttons*
  #(nil :left :right :middle :back :forward))

(defvar *key-codes* (make-array 125 :initial-element nil))

(loop for (code key) in
      '(;; Letters
        (1 :a) (2 :b) (3 :c) (4 :d) (5 :e) (6 :f) (7 :g)
        (8 :h) (9 :i) (10 :j) (11 :k) (12 :l) (13 :m) (14 :n)
        (15 :o) (16 :p) (17 :q) (18 :r) (19 :s) (20 :t) (21 :u)
        (22 :v) (23 :w) (24 :x) (25 :y) (26 :z)
        ;; Digits
        (30 :0) (31 :1) (32 :2) (33 :3) (34 :4)
        (35 :5) (36 :6) (37 :7) (38 :8) (39 :9)
        ;; Common
        (40 :space) (41 :enter) (42 :escape) (43 :backspace) (44 :tab)
        (45 :delete) (46 :insert) (47 :home) (48 :end)
        (49 :page-up) (50 :page-down)
        ;; Punctuation
        (55 :comma) (56 :period) (57 :slash) (58 :semicolon) (59 :quote)
        (60 :bracket-left) (61 :bracket-right) (62 :backslash)
        (63 :minus) (64 :equal) (65 :backquote)
        ;; Arrows
        (80 :up) (81 :down) (82 :left) (83 :right)
        ;; Modifiers
        (90 :shift-left) (91 :shift-right)
        (92 :control-left) (93 :control-right)
        (94 :alt-left) (95 :alt-right)
        (96 :super-left) (97 :super-right)
        ;; F-keys
        (100 :f1) (101 :f2) (102 :f3) (103 :f4) (104 :f5) (105 :f6)
        (106 :f7) (107 :f8) (108 :f9) (109 :f10) (110 :f11) (111 :f12)
        ;; Misc
        (120 :caps-lock) (121 :num-lock) (122 :scroll-lock)
        (123 :print-screen) (124 :pause))
      do (setf (aref *key-codes* code) key))

;;; Polling
(defparameter *input-buf* (%input-buf-ptr))
(defvar *read-index* 0)

(defun poll-events ()
  "Read all pending input events from the ring buffer.
Returns a list of (type key mods x y) lists with keyword symbols."
  (let ((write (%input-write-index))
        (read *read-index*)
        (event-size (cffi:foreign-type-size '(:struct input-event)))
        (events nil))
    (loop while (/= read write) do
      (let ((ptr (cffi:inc-pointer *input-buf* (* (mod read 256) event-size))))
        (cffi:with-foreign-slots ((event-type modifiers code x y) ptr (:struct input-event))
          (push (list (aref *event-types* event-type)
                      (if (<= event-type 2)
                          (aref *key-codes* code)
                          (when (< code (length *mouse-buttons*))
                            (aref *mouse-buttons* code)))
                      modifiers x y)
                events)))
      (setf read (ldb (byte 32 0) (1+ read))))
    (setf *read-index* read)
    (%input-set-read-index read)
    (nreverse events)))

;;; Input
(defcfun "set_input_callback" %set-input-callback :void (cb :pointer))

(defmacro handle-input ((type key mods x y) &body body)
  "Define the input handler. BODY runs once per input event with TYPE, KEY as
keywords, MODS as modifier bits, and X Y as floats. Replaces any previous handler."
  (let ((ev (gensym "EV")))
    `(progn
       (cffi:defcallback input-tick :void ()
         (dolist (,ev (poll-events))
           (destructuring-bind (,type ,key ,mods ,x ,y) ,ev
             (declare (ignorable ,type ,key ,mods ,x ,y))
             ,@body)))
       (%set-input-callback (cffi:callback input-tick)))))


