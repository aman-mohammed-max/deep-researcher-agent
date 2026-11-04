// SPDX-FileCopyrightText: Copyright (c) 2025-2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

/**
 * UI Adapters - Medusa UI Component Implementation
 *
 * Implements the UI adapter layer powered by Medusa UI (@medusajs/ui and @medusajs/icons)
 * following https://docs.medusajs.com/ui design system principles.
 *
 * All features import their UI primitives from here, ensuring consistent,
 * beautiful, minimalist aesthetic across the entire application.
 */

'use client'

import React, { type FC, forwardRef } from 'react'
import {
  Button as MedusaButton,
  Badge as MedusaBadge,
  Input as MedusaInput,
  Textarea as MedusaTextarea,
  Switch as MedusaSwitch,
  Checkbox as MedusaCheckbox,
  Tabs as MedusaTabs,
  Tooltip as MedusaTooltip,
  Divider as MedusaDivider,
  Table as MedusaTable,
  Skeleton as MedusaSkeleton,
  StatusBadge as MedusaStatusBadge,
} from '@medusajs/ui'
import { TooltipProvider as RadixTooltipProvider } from '@radix-ui/react-tooltip'
import { Spinner as MedusaSpinner, ChevronDown as MedusaChevronDown } from '@medusajs/icons'
import { cn } from '@/shared/lib/cn'

const CDN_VERSION = '3.8.0'
const CDN_BASE = `https://brand-assets.cne.ngc.nvidia.com/assets/icons/${CDN_VERSION}`

// ---------------------------------------------------------------------------
// Layout Components
// ---------------------------------------------------------------------------

export interface FlexProps extends React.HTMLAttributes<HTMLDivElement> {
  direction?: 'row' | 'col' | 'row-reverse' | 'col-reverse'
  align?: 'start' | 'center' | 'end' | 'baseline' | 'stretch'
  justify?: 'start' | 'center' | 'end' | 'between' | 'around' | 'evenly'
  gap?: string | number
  wrap?: boolean | 'wrap' | 'nowrap' | 'wrap-reverse'
  className?: string
  children?: React.ReactNode
}

export const Flex = forwardRef<HTMLDivElement, FlexProps>(
  (
    {
      direction = 'row',
      align = 'stretch',
      justify = 'start',
      gap,
      wrap,
      className,
      children,
      ...props
    },
    ref
  ) => {
    const dirClass =
      direction === 'col' ? 'flex-col' :
      direction === 'col-reverse' ? 'flex-col-reverse' :
      direction === 'row-reverse' ? 'flex-row-reverse' : 'flex-row'

    const alignClass =
      align === 'start' ? 'items-start' :
      align === 'center' ? 'items-center' :
      align === 'end' ? 'items-end' :
      align === 'baseline' ? 'items-baseline' : 'items-stretch'

    const justifyClass =
      justify === 'center' ? 'justify-center' :
      justify === 'end' ? 'justify-end' :
      justify === 'between' ? 'justify-between' :
      justify === 'around' ? 'justify-around' :
      justify === 'evenly' ? 'justify-evenly' : 'justify-start'

    const gapClass =
      gap === '0' || gap === 0 ? 'gap-0' :
      gap === '1' || gap === 1 ? 'gap-1' :
      gap === '2' || gap === 2 ? 'gap-2' :
      gap === '3' || gap === 3 ? 'gap-3' :
      gap === '4' || gap === 4 ? 'gap-4' :
      gap === '6' || gap === 6 ? 'gap-6' :
      gap === '8' || gap === 8 ? 'gap-8' :
      gap === 'density-sm' ? 'gap-1.5' :
      gap === 'density-md' ? 'gap-2.5' :
      gap === 'density-lg' ? 'gap-3' :
      gap === 'density-xl' ? 'gap-4' :
      typeof gap === 'string' && gap.startsWith('gap-') ? gap : ''

    const wrapClass =
      wrap === true || wrap === 'wrap' ? 'flex-wrap' : wrap === 'wrap-reverse' ? 'flex-wrap-reverse' : ''

    return (
      <div
        ref={ref}
        data-testid={(props as any)['data-testid'] || 'nv-flex'}
        className={cn('flex', dirClass, alignClass, justifyClass, gapClass, wrapClass, className)}
        {...props}
      >
        {children}
      </div>
    )
  }
)
Flex.displayName = 'Flex'

export interface StackProps extends React.HTMLAttributes<HTMLDivElement> {
  direction?: 'col' | 'row'
  align?: 'start' | 'center' | 'end' | 'baseline' | 'stretch'
  gap?: string | number
  className?: string
  children?: React.ReactNode
}

export const Stack: FC<StackProps> = ({ direction = 'col', align = 'stretch', gap = '4', className, children, ...props }) => {
  return (
    <Flex direction={direction} align={align} gap={gap} className={className} {...props}>
      {children}
    </Flex>
  )
}

export const Grid: FC<React.HTMLAttributes<HTMLDivElement>> = ({ className, children, ...props }) => (
  <div className={cn('grid', className)} {...props}>
    {children}
  </div>
)

export const Block: FC<React.HTMLAttributes<HTMLDivElement>> = ({ className, children, ...props }) => (
  <div className={cn('block', className)} {...props}>
    {children}
  </div>
)

export const Inline: FC<React.HTMLAttributes<HTMLSpanElement>> = ({ className, children, ...props }) => (
  <span className={cn('inline-flex items-center', className)} {...props}>
    {children}
  </span>
)

export const Group = Block

export interface DividerProps {
  orientation?: 'horizontal' | 'vertical'
  className?: string
}

export const Divider: FC<DividerProps> = ({ orientation = 'horizontal', className }) => {
  if (orientation === 'vertical') {
    return (
      <div
        className={cn('inline-block h-4 w-[1px] bg-ui-border-base mx-2 self-center', className)}
        role="separator"
      />
    )
  }
  return <MedusaDivider className={cn('my-2 border-ui-border-base', className)} />
}

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

export interface TextProps extends React.HTMLAttributes<HTMLElement> {
  kind?: string
  as?: any
  asChild?: boolean
  className?: string
  children?: React.ReactNode
}

export const Text = forwardRef<HTMLElement, TextProps>(
  ({ kind = 'body/regular/md', as, asChild, className, children, ...props }, ref) => {
    let defaultTag: any = 'p'
    let typographyClass = 'txt-medium text-ui-fg-base'

    if (kind.startsWith('h1')) {
      defaultTag = 'h1'
      typographyClass = 'h1-core font-semibold text-ui-fg-base tracking-tight'
    } else if (kind.startsWith('h2')) {
      defaultTag = 'h2'
      typographyClass = 'h2-core font-semibold text-ui-fg-base'
    } else if (kind.startsWith('h3')) {
      defaultTag = 'h3'
      typographyClass = 'h3-core font-medium text-ui-fg-base'
    } else if (kind.includes('label')) {
      defaultTag = 'span'
      if (kind.includes('bold') || kind.includes('semibold')) {
        typographyClass =
          kind.includes('lg')
            ? 'txt-compact-large-plus font-semibold text-ui-fg-base'
            : kind.includes('sm') || kind.includes('xs')
              ? 'txt-compact-small-plus font-medium text-ui-fg-base'
              : 'txt-compact-medium-plus font-medium text-ui-fg-base'
      } else {
        typographyClass =
          kind.includes('sm') || kind.includes('xs')
            ? 'txt-compact-small text-ui-fg-subtle'
            : 'txt-compact-medium text-ui-fg-base'
      }
    } else if (kind.includes('caption')) {
      defaultTag = 'span'
      typographyClass = 'txt-xsmall text-ui-fg-muted'
    } else if (kind.includes('sm') || kind.includes('xs')) {
      typographyClass = 'txt-small text-ui-fg-subtle'
    }

    if (asChild && React.isValidElement(children)) {
      return React.cloneElement(children as React.ReactElement<any>, {
        className: cn(typographyClass, (children.props as any).className, className),
        'data-testid': (props as any)['data-testid'] || (children.props as any)['data-testid'] || 'nv-text',
        ...props,
      })
    }

    const Tag = as || defaultTag
    return (
      <Tag
        ref={ref}
        data-testid={(props as any)['data-testid'] || 'nv-text'}
        className={cn(typographyClass, className)}
        {...props}
      >
        {children}
      </Tag>
    )
  }
)
Text.displayName = 'Text'

// ---------------------------------------------------------------------------
// Form Controls
// ---------------------------------------------------------------------------

export interface ButtonProps extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, 'color'> {
  kind?: 'primary' | 'secondary' | 'tertiary' | 'destructive'
  variant?: 'primary' | 'secondary' | 'transparent' | 'danger'
  color?: 'primary' | 'secondary' | 'danger' | 'brand'
  size?: 'tiny' | 'small' | 'medium' | 'large' | 'base' | 'xlarge'
  isLoading?: boolean
  asChild?: boolean
  children?: React.ReactNode
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      kind,
      variant,
      color,
      size = 'base',
      isLoading,
      className,
      children,
      ...props
    },
    ref
  ) => {
    const resolvedVariant: 'primary' | 'secondary' | 'transparent' | 'danger' =
      variant ||
      (kind === 'tertiary' ? 'transparent' :
       kind === 'destructive' || color === 'danger' ? 'danger' :
       kind === 'secondary' ? 'secondary' : 'primary')

    const resolvedSize =
      size === 'tiny' || size === 'small' ? 'small' :
      size === 'large' ? 'large' :
      size === 'xlarge' ? 'xlarge' : 'base'

    const isBrandGreen = color === 'brand' || className?.includes('brand')

    return (
      <MedusaButton
        ref={ref}
        variant={resolvedVariant}
        size={resolvedSize}
        isLoading={isLoading}
        className={cn(
          isBrandGreen && '!bg-[#76b900] hover:!bg-[#659f00] !text-black font-semibold shadow-buttons-inverted',
          className
        )}
        {...props}
      >
        {children}
      </MedusaButton>
    )
  }
)
Button.displayName = 'Button'

export const ButtonGroup: FC<React.HTMLAttributes<HTMLDivElement>> = ({ className, children, ...props }) => (
  <div className={cn('inline-flex rounded-md shadow-buttons-neutral', className)} role="group" {...props}>
    {children}
  </div>
)

export const TextInput = MedusaInput

export interface TextAreaProps extends Omit<React.TextareaHTMLAttributes<HTMLTextAreaElement>, 'size'> {
  value?: string
  onValueChange?: (value: string) => void
  resizeable?: string | boolean
  size?: string
  rows?: number
  className?: string
}

export const TextArea = forwardRef<HTMLTextAreaElement, TextAreaProps>(
  ({ value, onValueChange, onChange, className, resizeable, size: _size, ...props }, ref) => {
    const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
      onChange?.(e)
      onValueChange?.(e.target.value)
    }

    return (
      <MedusaTextarea
        ref={ref}
        value={value}
        onChange={handleChange}
        className={cn(
          resizeable === 'none' || resizeable === false ? 'resize-none' : 'resize-y',
          className
        )}
        {...props}
      />
    )
  }
)
TextArea.displayName = 'TextArea'

export interface SwitchProps {
  checked?: boolean
  defaultChecked?: boolean
  onCheckedChange?: (checked: boolean) => void
  onChange?: (e: any) => void
  disabled?: boolean
  size?: string
  className?: string
  'aria-label'?: string
  id?: string
}

export const Switch: FC<SwitchProps> = ({
  checked,
  defaultChecked,
  onCheckedChange,
  onChange,
  disabled,
  size: _size,
  className,
  ...props
}) => {
  const handleChange = (val: boolean) => {
    onCheckedChange?.(val)
    onChange?.({ target: { checked: val } } as any)
  }
  return (
    <MedusaSwitch
      checked={checked}
      defaultChecked={defaultChecked}
      onCheckedChange={handleChange}
      disabled={disabled}
      className={className}
      {...props}
    />
  )
}

export interface CheckboxProps {
  checked?: boolean | 'indeterminate'
  defaultChecked?: boolean
  onCheckedChange?: (checked: boolean | 'indeterminate') => void
  disabled?: boolean
  className?: string
  id?: string
  'aria-label'?: string
}

export const Checkbox: FC<CheckboxProps> = ({
  checked,
  defaultChecked,
  onCheckedChange,
  disabled,
  className,
  ...props
}) => {
  return (
    <MedusaCheckbox
      checked={checked}
      defaultChecked={defaultChecked}
      onCheckedChange={onCheckedChange}
      disabled={disabled}
      className={className}
      {...props}
    />
  )
}

export interface SelectItem {
  value: string
  children?: React.ReactNode
  label?: string
}

export interface SelectProps {
  value: string
  onValueChange: (val: string) => void
  items?: SelectItem[]
  side?: string
  className?: string
  children?: React.ReactNode
}

export const Select: FC<SelectProps> = ({ value, onValueChange, items, className, children }) => {
  const [open, setOpen] = React.useState(false)
  const selectedItem = items?.find((it) => it.value === value)
  const selectedLabel = typeof selectedItem?.children === 'string'
    ? selectedItem.children
    : selectedItem?.label || selectedItem?.value || value
  const containerRef = React.useRef<HTMLDivElement>(null)

  React.useEffect(() => {
    if (!open) return
    const handleClick = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false)
      }
    }
    document.addEventListener('mousedown', handleClick)
    return () => document.removeEventListener('mousedown', handleClick)
  }, [open])

  return (
    <div ref={containerRef} className={cn('relative inline-block w-full', className)}>
      <button
        type="button"
        role="combobox"
        aria-expanded={open}
        aria-haspopup="listbox"
        onClick={() => setOpen(!open)}
        className="w-full flex items-center justify-between rounded-md border border-ui-border-base bg-ui-bg-field px-3 py-1.5 txt-compact-small text-ui-fg-base shadow-borders-base outline-none transition-fg hover:bg-ui-bg-field-hover focus:shadow-borders-interactive-with-active cursor-pointer"
      >
        <span>{selectedLabel}</span>
        <MedusaChevronDown className={cn('h-4 w-4 text-ui-fg-muted transition-transform duration-200', open && 'rotate-180')} />
      </button>

      {open && (
        <div
          role="listbox"
          className="absolute z-50 mt-1 w-full rounded-md border border-ui-border-base bg-ui-bg-base p-1 shadow-elevation-flyout animate-in fade-in"
        >
          {items?.map((item) => {
            const isSelected = item.value === value
            return (
              <div
                key={item.value}
                role="option"
                aria-selected={isSelected}
                onClick={() => {
                  onValueChange(item.value)
                  setOpen(false)
                }}
                className={cn(
                  'rounded px-2.5 py-1.5 txt-compact-small text-ui-fg-base transition-fg cursor-pointer flex items-center justify-between',
                  isSelected ? 'bg-ui-bg-base-hover font-medium' : 'hover:bg-ui-bg-base-hover'
                )}
              >
                <span>{typeof item.children === 'string' ? item.children : item.label || item.value}</span>
              </div>
            )
          })}
          {children}
        </div>
      )}
    </div>
  )
}

export interface UploadProps {
  accept?: string
  multiple?: boolean
  value?: any[]
  onValueChange?: (files: any) => void
  disabled?: boolean
  listKind?: string
  renderInput?: (trigger: React.ReactNode) => React.ReactNode
  children?: React.ReactNode
  className?: string
}

export const Upload: FC<UploadProps> = ({
  accept,
  multiple,
  value,
  onValueChange,
  disabled,
  renderInput,
  children,
  className,
}) => {
  const inputRef = React.useRef<HTMLInputElement>(null)

  const handleFiles = (fileList: FileList | null) => {
    if (!fileList || fileList.length === 0) return
    const files = Array.from(fileList).map((f) => ({
      id: `${f.name}-${Date.now()}-${Math.random()}`,
      file: f,
      status: 'uploading',
    }))
    onValueChange?.(files)
  }

  const trigger = (
    <div
      onClick={() => !disabled && inputRef.current?.click()}
      onDragOver={(e) => {
        e.preventDefault()
        e.stopPropagation()
      }}
      onDrop={(e) => {
        e.preventDefault()
        e.stopPropagation()
        if (!disabled) handleFiles(e.dataTransfer.files)
      }}
      className={cn(
        'group flex flex-col items-center justify-center rounded-xl border-2 border-dashed border-ui-border-strong bg-ui-bg-field p-6 text-center transition-fg hover:border-ui-border-interactive hover:bg-ui-bg-field-hover cursor-pointer',
        disabled && 'cursor-not-allowed opacity-50',
        className
      )}
    >
      <input
        ref={inputRef}
        type="file"
        data-testid="nv-upload-input-element"
        accept={accept}
        multiple={multiple}
        disabled={disabled}
        className="hidden"
        onChange={(e) => handleFiles(e.target.files)}
      />
      <div className="txt-compact-small text-ui-fg-subtle">{children}</div>
    </div>
  )

  const content = renderInput ? <>{renderInput(trigger)}</> : trigger

  return (
    <div className="flex flex-col gap-3">
      {content}
      {value && value.length > 0 && (
        <div className="flex flex-col gap-2">
          {value.map((item) => (
            <div
              key={item.id}
              className="flex items-center justify-between p-2.5 rounded-lg bg-ui-bg-subtle border border-ui-border-base text-xs"
            >
              <div className="flex items-center gap-2 min-w-0">
                <span className="font-medium text-ui-fg-base truncate">{item.file?.name}</span>
                {item.errorMessage && (
                  <span className="text-ui-tag-red-text text-[11px] truncate">({item.errorMessage})</span>
                )}
              </div>
              <span className="text-ui-fg-subtle capitalize shrink-0">{item.status}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

export const FormField: FC<{ slotLabel?: React.ReactNode; children?: React.ReactNode; className?: string }> = ({
  slotLabel,
  children,
  className,
}) => {
  return (
    <div className={cn('flex flex-col gap-1.5', className)}>
      {slotLabel && <label className="txt-compact-small-plus text-ui-fg-base font-medium">{slotLabel}</label>}
      {children}
    </div>
  )
}

export const RadioGroup = Block
export const Slider = Block
export const Combobox = Block
export const DatePicker = Block

// ---------------------------------------------------------------------------
// Navigation
// ---------------------------------------------------------------------------

export const AppBar: FC<React.HTMLAttributes<HTMLElement>> = ({ className, children, ...props }) => (
  <header
    className={cn(
      'h-[var(--header-height)] border-b border-ui-border-base bg-ui-bg-base/80 backdrop-blur-md px-4 flex items-center justify-between',
      className
    )}
    {...props}
  >
    {children}
  </header>
)

export const Tabs = MedusaTabs
export const VerticalNav = Block
export const HorizontalNav = Block
export const Breadcrumbs = Block
export const Pagination = Block

// ---------------------------------------------------------------------------
// Feedback
// ---------------------------------------------------------------------------

export const Spinner: FC<{ className?: string; size?: 'tiny' | 'small' | 'medium' | 'large' | string; 'aria-label'?: string }> = ({
  className,
  size = 'small',
  'aria-label': ariaLabel = 'Loading',
  ...props
}) => {
  const sizeClass = size === 'large' ? 'w-8 h-8' : size === 'medium' ? 'w-6 h-6' : 'w-4 h-4'
  return (
    <MedusaSpinner
      className={cn('animate-spin text-ui-fg-interactive', sizeClass, className)}
      role="status"
      aria-label={ariaLabel}
      data-src={`${CDN_BASE}/line/circle-3-q.svg`}
      {...props}
    />
  )
}

export interface ProgressBarProps extends React.HTMLAttributes<HTMLDivElement> {
  value?: number
  max?: number
  'aria-label'?: string
}

export const ProgressBar: FC<ProgressBarProps> = ({
  value = 0,
  max = 100,
  'aria-label': ariaLabel,
  className,
  ...props
}) => {
  const pct = Math.min(100, Math.max(0, (value / max) * 100))
  return (
    <div
      role="progressbar"
      aria-valuenow={value}
      aria-valuemin={0}
      aria-valuemax={max}
      aria-label={ariaLabel}
      className={cn('h-1.5 w-full overflow-hidden rounded-full bg-ui-bg-field border border-ui-border-base', className)}
      {...props}
    >
      <div
        className="h-full bg-ui-bg-interactive transition-all duration-300"
        style={{ width: `${pct}%` }}
      />
    </div>
  )
}

export const Skeleton = MedusaSkeleton
export const Toast = Block
export const Notification = Block

export interface BannerProps extends React.HTMLAttributes<HTMLDivElement> {
  kind?: 'info' | 'success' | 'warning' | 'danger' | 'header' | 'inline' | string
  status?: 'info' | 'success' | 'warning' | 'danger' | 'error' | string
  slotHeading?: React.ReactNode
  slotSubheading?: React.ReactNode
  slotActions?: React.ReactNode
  actionsPosition?: string
  onClose?: () => void
  children?: React.ReactNode
  className?: string
}

export const Banner: FC<BannerProps> = ({
  kind = 'info',
  status,
  slotHeading,
  slotSubheading,
  slotActions,
  actionsPosition: _actionsPosition,
  onClose,
  children,
  className,
  ...props
}) => {
  const effectiveKind = status === 'error' ? 'danger' : (status || kind)
  const colorMap: Record<string, string> = {
    info: 'bg-ui-bg-field border-ui-border-base text-ui-fg-base',
    header: 'bg-ui-bg-field border-ui-border-base text-ui-fg-base',
    inline: 'bg-ui-bg-field border-ui-border-base text-ui-fg-base',
    success: 'bg-ui-tag-green-bg border-ui-tag-green-border text-ui-tag-green-text',
    warning: 'bg-ui-tag-orange-bg border-ui-tag-orange-border text-ui-tag-orange-text',
    danger: 'bg-ui-tag-red-bg border-ui-tag-red-border text-ui-tag-red-text',
  }

  return (
    <div
      data-testid={(props as any)['data-testid'] || 'nv-banner'}
      className={cn(
        'nv-banner banner rounded-lg border p-3 txt-compact-small flex flex-col gap-2',
        colorMap[effectiveKind] || colorMap.info,
        className
      )}
      role="alert"
      {...props}
    >
      {(slotHeading || slotSubheading || slotActions || onClose) && (
        <div className="flex items-center justify-between font-medium">
          <div className="flex flex-col gap-0.5">
            {slotHeading && <div className="text-ui-fg-base font-semibold">{slotHeading}</div>}
            {slotSubheading && <div className="text-ui-fg-subtle text-xs">{slotSubheading}</div>}
          </div>
          <div className="flex items-center gap-2">
            {slotActions && <div>{slotActions}</div>}
            {onClose && (
              <button
                type="button"
                onClick={onClose}
                className="rounded p-0.5 text-ui-fg-muted hover:text-ui-fg-base transition-colors"
                aria-label="Dismiss banner"
              >
                ✕
              </button>
            )}
          </div>
        </div>
      )}
      {children}
    </div>
  )
}

export const StatusIndicator = MedusaStatusBadge
export const StatusMessage = Banner

// ---------------------------------------------------------------------------
// Overlays
// ---------------------------------------------------------------------------

export interface ModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  slotHeading?: React.ReactNode
  slotFooter?: React.ReactNode
  children?: React.ReactNode
  className?: string
}

const ModalContext = React.createContext<{ close: () => void }>({ close: () => {} })

export const Modal: FC<ModalProps> = ({ open, onOpenChange, slotHeading, slotFooter, children, className }) => {
  if (!open) return null
  const close = () => onOpenChange(false)

  return (
    <ModalContext.Provider value={{ close }}>
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
        {/* Backdrop */}
        <div
          className="fixed inset-0 bg-ui-bg-overlay backdrop-blur-sm transition-opacity"
          onClick={close}
          aria-hidden="true"
        />

        {/* Dialog Content */}
        <div
          className={cn(
            'relative z-10 w-full max-w-lg rounded-xl border border-ui-border-base bg-ui-bg-base p-6 shadow-elevation-modal duration-200 animate-in fade-in zoom-in-95',
            className
          )}
          role="dialog"
          aria-modal="true"
        >
          {slotHeading && (
            <div className="mb-4 pb-3 border-b border-ui-border-base flex items-center justify-between font-medium text-ui-fg-base">
              {slotHeading}
            </div>
          )}

          <div className="text-ui-fg-subtle txt-medium my-2">
            {children}
          </div>

          {slotFooter && (
            <div className="mt-6 pt-3 border-t border-ui-border-base flex items-center justify-end gap-2">
              {slotFooter}
            </div>
          )}
        </div>
      </div>
    </ModalContext.Provider>
  )
}

export const ModalCloseButton: FC<ButtonProps> = ({ children = 'Close', onClick, kind = 'tertiary', ...props }) => {
  const { close } = React.useContext(ModalContext)
  return (
    <Button
      kind={kind}
      onClick={(e) => {
        onClick?.(e)
        close()
      }}
      {...props}
    >
      {children}
    </Button>
  )
}

export interface PopoverProps {
  open?: boolean
  onOpenChange?: (open: boolean) => void
  side?: 'top' | 'right' | 'bottom' | 'left'
  align?: 'start' | 'center' | 'end'
  slotContent?: React.ReactNode
  children?: React.ReactNode
  className?: string
}

export const Popover: FC<PopoverProps> = ({
  open,
  onOpenChange,
  side: _side = 'bottom',
  align = 'end',
  slotContent,
  children,
  className,
}) => {
  const [internalOpen, setInternalOpen] = React.useState(false)
  const isControlled = open !== undefined
  const isOpen = isControlled ? open : internalOpen
  const setIsOpen = React.useCallback(
    (next: boolean) => {
      if (!isControlled) setInternalOpen(next)
      onOpenChange?.(next)
    },
    [isControlled, onOpenChange]
  )

  const containerRef = React.useRef<HTMLDivElement>(null)

  React.useEffect(() => {
    if (!isOpen) return
    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [isOpen, setIsOpen])

  return (
    <div ref={containerRef} className="relative inline-block">
      <div onClick={() => setIsOpen(!isOpen)}>{children}</div>
      {isOpen && (
        <div
          data-testid="nv-popover-content"
          className={cn(
            'nv-popover-content bg-surface-base absolute z-50 mt-2 min-w-[200px] rounded-lg border border-ui-border-base bg-ui-bg-base p-3 shadow-elevation-flyout animate-in fade-in zoom-in-95',
            align === 'end' ? 'right-0' : 'left-0',
            className
          )}
        >
          {slotContent}
        </div>
      )}
    </div>
  )
}

export const Tooltip: FC<{ children: React.ReactNode; content?: React.ReactNode; className?: string }> = ({
  children,
  content,
  ...props
}) => {
  if (!content) return <>{children}</>
  return (
    <RadixTooltipProvider delayDuration={100}>
      <MedusaTooltip content={content} {...props}>
        {children}
      </MedusaTooltip>
    </RadixTooltipProvider>
  )
}

export interface SidePanelProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  dock?: 'left' | 'right'
  side?: 'left' | 'right' | string
  bordered?: boolean
  closeOnClickOutside?: boolean
  size?: 'small' | 'medium' | 'large'
  slotHeading?: React.ReactNode
  slotFooter?: React.ReactNode
  children?: React.ReactNode
  className?: string
}

export const SidePanel: FC<SidePanelProps> = ({
  open,
  onOpenChange,
  dock,
  side = 'right',
  bordered,
  closeOnClickOutside: _closeOnClickOutside,
  size = 'medium',
  slotHeading,
  slotFooter,
  children,
  className,
}) => {
  if (!open) return null

  const effectiveSide = dock || side
  const widthClass = size === 'large' ? 'w-[520px]' : size === 'small' ? 'w-[320px]' : 'w-[400px]'

  return (
    <div
      className={cn(
        'relative flex h-full flex-col border-ui-border-base bg-ui-bg-subtle shadow-elevation-flyout z-30 transition-all',
        effectiveSide === 'right' ? 'border-l' : 'border-r',
        bordered && 'border-ui-border-strong',
        widthClass,
        className
      )}
    >
      {slotHeading && (
        <div className="flex items-center justify-between border-b border-ui-border-base px-6 py-4 font-medium text-ui-fg-base">
          {slotHeading}
          <button
            type="button"
            onClick={() => onOpenChange(false)}
            className="rounded-md p-1 text-ui-fg-muted hover:bg-ui-bg-base-hover hover:text-ui-fg-base transition-colors"
            aria-label="Close panel"
          >
            ✕
          </button>
        </div>
      )}

      <div className="flex-1 overflow-y-auto px-6 py-4 text-ui-fg-base">
        {children}
      </div>

      {slotFooter && (
        <div className="border-t border-ui-border-base px-6 py-4 bg-ui-bg-base/40">
          {slotFooter}
        </div>
      )}
    </div>
  )
}

export const Panel = SidePanel
export const Menu = Block
export const Dropdown = Block

// ---------------------------------------------------------------------------
// Data Display
// ---------------------------------------------------------------------------

export interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
  className?: string
  children?: React.ReactNode
}

export const Card = forwardRef<HTMLDivElement, CardProps>(({ className, children, ...props }, ref) => {
  return (
    <div
      ref={ref}
      className={cn(
        'rounded-lg bg-ui-bg-base border border-ui-border-base p-4 shadow-elevation-card-rest transition-fg hover:shadow-elevation-card-hover',
        className
      )}
      {...props}
    >
      {children}
    </div>
  )
})
Card.displayName = 'Card'

export const Table = MedusaTable
export const List = Block

export interface AvatarProps {
  src?: string
  fallback?: string
  size?: 'small' | 'medium' | 'large'
  className?: string
  alt?: string
}

export const Avatar: FC<AvatarProps> = ({ src, fallback = 'U', size = 'small', className }) => {
  const sizeClass =
    size === 'large' ? 'h-10 w-10 txt-compact-medium' :
    size === 'medium' ? 'h-8 w-8 txt-compact-small' : 'h-6 w-6 txt-compact-xsmall'
  return (
    <div
      className={cn(
        'relative inline-flex shrink-0 items-center justify-center rounded-full bg-ui-bg-component border border-ui-border-base overflow-hidden font-medium text-ui-fg-base',
        sizeClass,
        className
      )}
    >
      {src ? (
        <img src={src} alt={fallback} className="h-full w-full object-cover" />
      ) : (
        <span>{fallback}</span>
      )}
    </div>
  )
}

export interface BadgeProps extends React.HTMLAttributes<HTMLSpanElement> {
  kind?: 'neutral' | 'success' | 'warning' | 'danger' | 'info' | 'brand'
  color?: 'grey' | 'green' | 'blue' | 'red' | 'orange' | 'purple'
  size?: 'small' | 'base' | 'large'
  rounded?: 'base' | 'full'
  className?: string
  children?: React.ReactNode
}

export const Badge: FC<BadgeProps> = ({
  kind = 'neutral',
  color,
  size = 'small',
  rounded = 'base',
  className,
  children,
  ...props
}) => {
  const mappedColor: 'grey' | 'green' | 'blue' | 'red' | 'orange' | 'purple' =
    color ||
    (kind === 'success' || kind === 'brand' ? 'green' :
     kind === 'warning' ? 'orange' :
     kind === 'danger' ? 'red' :
     kind === 'info' ? 'blue' : 'grey')

  return (
    <MedusaBadge
      color={mappedColor}
      size={size === 'small' ? 'small' : 'base'}
      rounded={rounded}
      className={className}
      {...props}
    >
      {children}
    </MedusaBadge>
  )
}

export const Tag = Badge
export const Label = Text

export interface CodeSnippetProps {
  value?: string
  language?: string
  kind?: string
  collapsible?: boolean
  rows?: number
  className?: string
  children?: React.ReactNode
}

export const CodeSnippet: FC<CodeSnippetProps> = ({ value, language = 'text', className, children }) => {
  const code = (value ?? (typeof children === 'string' ? children : '')).trimEnd()
  return (
    <div className={cn('my-3 overflow-hidden rounded-lg border border-ui-border-base bg-ui-contrast-bg-base text-ui-contrast-fg-primary shadow-elevation-code-block', className)}>
      {language && (
        <div className="flex items-center justify-between border-b border-ui-contrast-border-bot px-4 py-2 txt-compact-small text-ui-contrast-fg-secondary font-mono">
          <span>{language}</span>
        </div>
      )}
      <pre className="overflow-x-auto p-4 txt-compact-small font-mono">
        <code>{code}</code>
      </pre>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Utility Components
// ---------------------------------------------------------------------------

export const Accordion = Block
export const Collapsible = Block

export interface AnchorProps extends React.AnchorHTMLAttributes<HTMLAnchorElement> {
  kind?: string
}

export const Anchor: FC<AnchorProps> = ({ kind: _kind, className, children, ...props }) => (
  <a className={cn('text-ui-fg-interactive hover:underline transition-colors', className)} {...props}>
    {children}
  </a>
)

export interface SegmentedControlItem {
  value: string
  children?: React.ReactNode
  label?: string
}

export interface SegmentedControlProps {
  value: string
  onValueChange: (val: string) => void
  items: SegmentedControlItem[]
  size?: 'small' | 'medium' | 'large'
  className?: string
}

export const SegmentedControl: FC<SegmentedControlProps> = ({
  value,
  onValueChange,
  items,
  size = 'medium',
  className,
}) => {
  return (
    <div
      role="radiogroup"
      className={cn(
        'inline-flex items-center gap-1 rounded-lg border border-ui-border-base bg-ui-bg-field p-1 shadow-borders-base',
        className
      )}
    >
      {items.map((item) => {
        const selected = item.value === value
        return (
          <button
            key={item.value}
            type="button"
            role="radio"
            aria-checked={selected}
            onClick={() => onValueChange(item.value)}
            className={cn(
              'rounded-md px-3 py-1 font-medium transition-fg text-center outline-none',
              size === 'small' ? 'txt-compact-xsmall' : 'txt-compact-small',
              selected
                ? 'bg-ui-bg-base text-ui-fg-base shadow-elevation-card-rest'
                : 'text-ui-fg-muted hover:text-ui-fg-base hover:bg-ui-bg-base-hover'
            )}
          >
            {item.children || item.label || item.value}
          </button>
        )
      })}
    </div>
  )
}

export interface AnimatedChevronProps {
  expanded?: boolean
  state?: 'open' | 'closed' | string
  className?: string
  size?: 'small' | 'medium' | 'large'
}

export const AnimatedChevron: FC<AnimatedChevronProps> = ({ expanded = false, state, className, size = 'small' }) => {
  const isExpanded = state ? state === 'open' : expanded
  const sizeClass = size === 'large' ? 'h-5 w-5' : size === 'medium' ? 'h-4 w-4' : 'h-3.5 w-3.5'
  return (
    <MedusaChevronDown
      className={cn('transition-transform duration-200 text-ui-fg-muted', sizeClass, isExpanded && 'rotate-180', className)}
    />
  )
}

// ---------------------------------------------------------------------------
// Theme
// ---------------------------------------------------------------------------

interface ThemeProviderProps {
  theme?: string
  global?: boolean
  defer?: boolean
  children?: React.ReactNode
}

export const ThemeProvider: FC<ThemeProviderProps> = ({ theme = 'light', children }) => {
  React.useEffect(() => {
    const root = document.documentElement
    if (theme === 'dark') {
      root.classList.add('dark', 'nv-dark')
      root.classList.remove('light', 'nv-light')
      root.setAttribute('data-mode', 'dark')
    } else {
      root.classList.add('light', 'nv-light')
      root.classList.remove('dark', 'nv-dark')
      root.setAttribute('data-mode', 'light')
    }
  }, [theme])

  return <>{children}</>
}

// ---------------------------------------------------------------------------
// Branding & Page
// ---------------------------------------------------------------------------

export { Logo } from './Logo'
export const PageHeader = Block
export const Hero = Block
