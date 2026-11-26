;;; interactive.el --- compmd-ts-mode against local builds, in place of the writer's config -*- lexical-binding: t; -*-

;;; Commentary:

;; `cargo make emacs-interactive [file]' builds what it needs and starts it:
;;
;;   emacs -nw -Q -l editors/emacs/test/interactive.el [file]
;;
;; Without a file, it opens a copy of the Author's Guide's containers example, in a folder of its own, to edit freely.
;; Eglot starts with each CompMD buffer.

;;; Code:

(require 'eglot)

(let* ((emacs (file-name-directory (directory-file-name (file-name-directory load-file-name))))
       (repository (file-name-directory (directory-file-name (file-name-directory (directory-file-name emacs))))))
  (load (expand-file-name "compmd-ts-mode.el" emacs) nil t)
  (setq treesit-extra-load-path (list (expand-file-name "target/emacs" repository)))
  (setq eglot-server-programs
        `((compmd-ts-mode ,(expand-file-name "target/debug/compmd-lsp" repository) "--stdio")))
  (add-hook 'compmd-ts-mode-hook #'eglot-ensure)
  (unless command-line-args-left
    (let ((copy (expand-file-name "garden.comp.md" (make-temp-file "compmd-emacs-" t))))
      (copy-file (expand-file-name "guides/authors/examples/containers/garden.comp.md" repository) copy)
      (add-hook 'emacs-startup-hook (lambda () (find-file copy))))))

;;; interactive.el ends here
