(in-package :lambda)

;; FFI
(cffi:defcfun ("lambda_window_width"  window-width)  :uint32)
(cffi:defcfun ("lambda_window_height" window-height) :uint32)

(defstruct node
  "One element in the UI tree."
  type          ; :vstack :hstack :rect :text :image
  content       ; string or nil (text content, image path)
  styles        ; plist (:color #xFF0000FF :size 24 ...)
  children      ; list of child nodes
  id            ; u32 GPU element ID, nil for layout-only nodes
  x y width height) ; computed by layout

(defstruct ui
  "Root node and name lookup table."
  root          ; root node (implicit vstack)
  names)        ; hash table: keyword -> node

(defun flatten-children (forms)
  "Flatten one level of nesting and remove nils from children."
  (mapcan (lambda (f)
            (cond ((node-p f) (list f))
                  ((listp f) (remove-if-not #'node-p f))))
          forms))

(defun expand-styles (styles)
  "Expand styles form: inline plist becomes (list ...), otherwise passed through."
  (if (and (listp styles) (keywordp (car styles)))
      `(list ,@styles)
      styles))

;; Element macros
(defmacro vstack (styles &body children)
  `(make-node :type :vstack
              :styles ,(expand-styles styles)
              :children (flatten-children (list ,@children))))

(defmacro hstack (styles &body children)
  `(make-node :type :hstack
              :styles ,(expand-styles styles)
              :children (flatten-children (list ,@children))))

(defmacro rect (styles &body children)
  `(make-node :type :rect
              :styles ,(expand-styles styles)
              :children (flatten-children (list ,@children))))

(defmacro text (content &optional styles)
  `(make-node :type :text
              :content ,content
              :styles ,(expand-styles styles)))

(defmacro image (path &optional styles)
  `(make-node :type :image
              :content ,path
              :styles ,(expand-styles styles)))

;; Element creation
(defun create-element (node)
  "Create the GPU element for a node. Returns the ID or nil."
  (let ((styles (node-styles node)))
    (case (node-type node)
      (:rect
        (let ((id (make-rect)))
          (let ((c (getf styles :color)))    (when c (rect-color id c)))
          (let ((r (getf styles :radius)))   (when r (rect-radius id r)))
          (let ((bw (getf styles :border-width)))
            (when bw (rect-border id bw (or (getf styles :border-color) #x000000FF))))
          id))
      (:text
        (let* ((size (or (getf styles :size) 16))
               (lh (or (getf styles :line-height) (* size 1.4)))
               (id (make-text size lh)))
          (when (node-content node) (text-set id (node-content node)))
          (let ((c (getf styles :color))) (when c (text-color id c)))
          id))
      (:image
        (when (node-content node)
          (make-image (node-content node))))
      (otherwise nil))))

(defun create-elements (ui node)
  "Walk the tree, create GPU elements, register :name entries."
  (setf (node-id node) (create-element node))
  (let ((name (getf (node-styles node) :name)))
    (when name (setf (gethash name (ui-names ui)) node)))
  (dolist (child (node-children node))
    (create-elements ui child)))

;; Layout
(defun vertical-p (type)
  (member type '(:vstack :rect)))

(defun resolve-width (node available)
  (let ((sw (getf (node-styles node) :width))
        (id (node-id node)))
    (cond
      ((numberp sw) sw)
      ((eq sw :fill) available)
      ((and (eq (node-type node) :text) id) (text-width id))
      ((and (eq (node-type node) :image) id) (image-width id))
      (t available))))

(defun resolve-height (node available)
  (let ((sh (getf (node-styles node) :height))
        (id (node-id node)))
    (cond
      ((numberp sh) sh)
      ((eq sh :fill) available)
      ((and (eq (node-type node) :text) id) (text-height id))
      ((and (eq (node-type node) :image) id) (image-height id))
      (t nil))))

(defun compute-layout (node x y available-w available-h)
  "Compute x, y, width, height for a node and all descendants."
  (let* ((styles (node-styles node))
         (padding (or (getf styles :padding) 0))
         (gap (or (getf styles :gap) 0))
         (w (resolve-width node available-w))
         (content-x (+ x padding))
         (content-y (+ y padding))
         (content-w (- w (* 2 padding)))
         (cursor-x content-x)
         (cursor-y content-y))

    (setf (node-x node) x
          (node-y node) y
          (node-width node) w)

    (let ((n (length (node-children node))))
      (loop for child in (node-children node)
            for i from 0
            do (if (vertical-p (node-type node))
                   (progn
                     (compute-layout child cursor-x cursor-y
                                     content-w (- available-h (- cursor-y y)))
                     (incf cursor-y (node-height child))
                     (when (< i (1- n)) (incf cursor-y gap)))
                   (progn
                     (compute-layout child cursor-x cursor-y
                                     (- content-w (- cursor-x content-x))
                                     (- available-h (* 2 padding)))
                     (incf cursor-x (node-width child))
                     (when (< i (1- n)) (incf cursor-x gap))))))

    (let ((h (resolve-height node available-h)))
      (setf (node-height node)
            (or h
                (+ (* 2 padding)
                   (if (node-children node)
                       (if (vertical-p (node-type node))
                           (- cursor-y content-y)
                           (loop for c in (node-children node)
                                 maximize (node-height c)))
                       0)))))))

(defun apply-layout (node)
  "Walk the tree, apply computed positions/sizes via FFI."
  (let ((id (node-id node)))
    (when id
      (let ((x (node-x node)) (y (node-y node))
            (w (node-width node)) (h (node-height node)))
        (case (node-type node)
          (:rect  (rect-position id x y) (rect-size id w h))
          (:text  (text-position id x y)
            (text-bounds id (truncate x) (truncate y)
                         (truncate (+ x w)) (truncate (+ y h))))
          (:image (image-position id x y) (image-size id w h))))))
  (dolist (child (node-children node))
    (apply-layout child)))

;; UI
(defun destroy-ui (ui)
  "Destroy all GPU elements in a UI tree."
  (labels ((walk (node)
             (when (node-id node)
               (case (node-type node)
                 (:rect  (rect-destroy (node-id node)))
                 (:text  (text-destroy (node-id node)))
                 (:image (image-destroy (node-id node)))))
             (dolist (child (node-children node))
               (walk child))))
    (walk (ui-root ui))))

(defun ui-ref (ui name)
  "Get the GPU element ID for a named element."
  (let ((node (gethash name (ui-names ui))))
    (when node (node-id node))))

(defun layout (ui)
  "Recompute layout and apply positions/sizes."
  (let* ((styles (node-styles (ui-root ui)))
         (x (or (getf styles :x) 0))
         (y (or (getf styles :y) 0))
         (w (or (getf styles :width) (window-width)))
         (h (or (getf styles :height) (window-height))))
    (compute-layout (ui-root ui) x y w h)
    (apply-layout (ui-root ui))))

(defun build-ui (root)
  "Create GPU elements and compute layout for a node tree."
  (let ((ui (make-ui :root root :names (make-hash-table :test #'eq))))
    (create-elements ui root)
    (layout ui)
    ui))

(defmacro defui (name styles &body body)
  "Define a UI tree with full Lisp expressiveness."
  `(progn
     (when (boundp ',name) (destroy-ui (symbol-value ',name)))
     (defparameter ,name
       (build-ui (vstack ,styles ,@body)))))
