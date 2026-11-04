// SPDX-FileCopyrightText: Copyright (c) 2025-2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

/**
 * Icon Adapters - Medusa UI Icons
 *
 * Uses @medusajs/icons for a clean, consistent, minimalist design language
 * aligned with https://docs.medusajs.com/ui.
 *
 * Features should import icons from '@/adapters/ui/icons'.
 */

'use client'

import React, { type FC } from 'react'
import {
  ChevronDown as MedusaChevronDown,
  ChevronLeft as MedusaChevronLeft,
  ChevronRight as MedusaChevronRight,
  ChevronUpMini as MedusaChevronUpMini,
  BarsThree as MedusaBarsThree,
  ArrowLeft as MedusaArrowLeft,
  ArrowRight as MedusaArrowRight,
  MagnifyingGlass as MedusaMagnifyingGlass,
  Plus as MedusaPlus,
  Minus as MedusaMinus,
  XMark as MedusaXMark,
  Check as MedusaCheck,
  PencilSquare as MedusaPencilSquare,
  SquareTwoStack as MedusaSquareTwoStack,
  ArrowDownTray as MedusaArrowDownTray,
  ArrowUpTray as MedusaArrowUpTray,
  Share as MedusaShare,
  ArrowUpRightOnBox as MedusaArrowUpRightOnBox,
  ArrowPath as MedusaArrowPath,
  PaperPlane as MedusaPaperPlane,
  ChatBubble as MedusaChatBubble,
  Envelope as MedusaEnvelope,
  InformationCircle as MedusaInformationCircle,
  ExclamationCircle as MedusaExclamationCircle,
  XCircle as MedusaXCircle,
  CheckCircleSolid as MedusaCheckCircleSolid,
  CheckCircle as MedusaCheckCircle,
  User as MedusaUser,
  Users as MedusaUsers,
  CogSixTooth as MedusaCogSixTooth,
  ArrowRightOnRectangle as MedusaArrowRightOnRectangle,
  Folder as MedusaFolder,
  Photo as MedusaPhoto,
  Code as MedusaCode,
  Sun as MedusaSun,
  Moon as MedusaMoon,
  Star as MedusaStar,
  Heart as MedusaHeart,
  Calendar as MedusaCalendar,
  Clock as MedusaClock,
  Sparkles as MedusaSparkles,
  DocumentText as MedusaDocumentText,
  Link as MedusaLink,
  Trash as MedusaTrash,
  GlobeEurope as MedusaGlobeEurope,
  BookOpen as MedusaBookOpen,
  QuestionMarkCircle as MedusaQuestionMarkCircle,
  LockClosedSolid as MedusaLockClosedSolid,
  Wrench as MedusaWrench,
  PaperClip as MedusaPaperClip,
  StopCircleSolid as MedusaStopCircleSolid,
  AiAssistent as MedusaAiAssistent,
  CircleDottedLine as MedusaCircleDottedLine,
  Spinner as MedusaSpinner,
  Adjustments as MedusaAdjustments,
  ChevronUpDown as MedusaChevronUpDown,
  BuildingStorefront as MedusaBuildingStorefront,
} from '@medusajs/icons'

const CDN_VERSION = '3.8.0'
const CDN_BASE = `https://brand-assets.cne.ngc.nvidia.com/assets/icons/${CDN_VERSION}`

export interface IconProps extends React.SVGProps<SVGSVGElement> {
  className?: string
  width?: number | string
  height?: number | string
  'aria-label'?: string
}

type MedusaIconComponent = React.ComponentType<{
  className?: string
  width?: number | string
  height?: number | string
  'aria-label'?: string
  'aria-hidden'?: boolean | 'true' | 'false'
  'data-src'?: string
  color?: string
  onClick?: React.MouseEventHandler<SVGElement>
}>

const createIcon = (
  Component: MedusaIconComponent,
  displayName: string,
  iconName?: string,
  extraClassName?: string
): FC<IconProps> => {
  const Icon: FC<IconProps> = ({ className, width, height, 'aria-label': ariaLabel, onClick, ...rest }) => {
    const combinedClassName = [extraClassName, className].filter(Boolean).join(' ')
    const dataSrc = iconName ? `${CDN_BASE}/line/${iconName}.svg` : undefined
    return (
      <Component
        width={width}
        height={height}
        className={combinedClassName || undefined}
        aria-hidden={!ariaLabel}
        aria-label={ariaLabel}
        data-src={dataSrc}
        onClick={onClick}
        {...rest}
      />
    )
  }
  Icon.displayName = displayName
  return Icon
}

// ---------------------------------------------------------------------------
// Navigation icons
// ---------------------------------------------------------------------------

export const ChevronDown = createIcon(MedusaChevronDown, 'ChevronDown', 'chevron-down')
export const ChevronUp = createIcon(MedusaChevronUpMini, 'ChevronUp', 'chevron-up')
export const ChevronLeft = createIcon(MedusaChevronLeft, 'ChevronLeft', 'chevron-left')
export const ChevronRight = createIcon(MedusaChevronRight, 'ChevronRight', 'chevron-right')
export const ArrowLeft = createIcon(MedusaArrowLeft, 'ArrowLeft', 'arrow-left')
export const ArrowRight = createIcon(MedusaArrowRight, 'ArrowRight', 'arrow-right')
export const Menu = createIcon(MedusaBarsThree, 'Menu', 'menu')

// ---------------------------------------------------------------------------
// Action icons
// ---------------------------------------------------------------------------

export const Search = createIcon(MedusaMagnifyingGlass, 'Search', 'magnifying-glass')
export const Plus = createIcon(MedusaPlus, 'Plus', 'add')
export const Minus = createIcon(MedusaMinus, 'Minus', 'subtract')
export const Close = createIcon(MedusaXMark, 'Close', 'close')
export const Check = createIcon(MedusaCheck, 'Check', 'check')
export const Edit = createIcon(MedusaPencilSquare, 'Edit', 'pencil')
export const Copy = createIcon(MedusaSquareTwoStack, 'Copy', 'copy-generic')
export const Download = createIcon(MedusaArrowDownTray, 'Download', 'download')
export const Upload = createIcon(MedusaArrowUpTray, 'Upload', 'upload')
export const Share = createIcon(MedusaShare, 'Share', 'share')
export const OpenExternal = createIcon(MedusaArrowUpRightOnBox, 'OpenExternal', 'open-external')
export const Refresh = createIcon(MedusaArrowPath, 'Refresh', 'refresh')

// ---------------------------------------------------------------------------
// Communication icons
// ---------------------------------------------------------------------------

export const Send = createIcon(MedusaPaperPlane, 'Send', 'forward')
export const Chat = createIcon(MedusaChatBubble, 'Chat', 'chat-single')
export const Mail = createIcon(MedusaEnvelope, 'Mail', 'envelope')

// ---------------------------------------------------------------------------
// Status icons
// ---------------------------------------------------------------------------

export const Info = createIcon(MedusaInformationCircle, 'Info', 'info-circle')
export const Warning = createIcon(MedusaExclamationCircle, 'Warning', 'warning')
export const Error = createIcon(MedusaXCircle, 'Error', 'cancel')
export const Success = createIcon(MedusaCheckCircle, 'Success', 'check')

// ---------------------------------------------------------------------------
// User icons
// ---------------------------------------------------------------------------

export const User = createIcon(MedusaUser, 'User', 'profile')
export const Users = createIcon(MedusaUsers, 'Users', 'profile-group')
export const Settings = createIcon(MedusaCogSixTooth, 'Settings', 'cog')
export const Logout = createIcon(MedusaArrowRightOnRectangle, 'Logout', 'exit')

// ---------------------------------------------------------------------------
// Content icons
// ---------------------------------------------------------------------------

export const Folder = createIcon(MedusaFolder, 'Folder', 'folder-closed')
export const Image = createIcon(MedusaPhoto, 'Image', 'image')
export const Code = createIcon(MedusaCode, 'Code', 'code')

// ---------------------------------------------------------------------------
// Theme icons
// ---------------------------------------------------------------------------

export const Sun = createIcon(MedusaSun, 'Sun', 'sun-high')
export const Moon = createIcon(MedusaMoon, 'Moon', 'moon')

// ---------------------------------------------------------------------------
// Misc icons
// ---------------------------------------------------------------------------

export const Star = createIcon(MedusaStar, 'Star', 'star')
export const Heart = createIcon(MedusaHeart, 'Heart', 'heart')
export const Home = createIcon(MedusaBuildingStorefront, 'Home', 'home')
export const Calendar = createIcon(MedusaCalendar, 'Calendar', 'calendar')
export const Clock = createIcon(MedusaClock, 'Clock', 'clock')
export const Filter = createIcon(MedusaAdjustments, 'Filter', 'filter')
export const Sort = createIcon(MedusaChevronUpDown, 'Sort', 'sort')
export const Wand = createIcon(MedusaSparkles, 'Wand', 'wand')

// ---------------------------------------------------------------------------
// Aliased / additional icons
// ---------------------------------------------------------------------------

export const Document = createIcon(MedusaDocumentText, 'Document', 'document')
export const DocumentCheckmark = createIcon(MedusaCheckCircleSolid, 'DocumentCheckmark', 'document-checkmark')
export const Link = createIcon(MedusaLink, 'Link', 'link')
export const Trash = createIcon(MedusaTrash, 'Trash', 'trash')
export const Globe = createIcon(MedusaGlobeEurope, 'Globe', 'world')
export const Book = createIcon(MedusaBookOpen, 'Book', 'book')
export const Help = createIcon(MedusaQuestionMarkCircle, 'Help', 'help-circle')
export const Lock = createIcon(MedusaLockClosedSolid, 'Lock', 'lock-closed')
export const Plug = createIcon(MedusaLink, 'Plug', 'plug-recepticle')
export const Wrench = createIcon(MedusaWrench, 'Wrench', 'wrench')
export const ChatMessage = createIcon(MedusaChatBubble, 'ChatMessage', 'chat-message')
export const SelectEllipse = createIcon(MedusaCircleDottedLine, 'SelectEllipse', 'select-ellipse')
export const Retry = createIcon(MedusaArrowPath, 'Retry', 'retry')
export const Cancel = createIcon(MedusaXCircle, 'Cancel', 'cancel')

// ---------------------------------------------------------------------------
// GUI icons
// ---------------------------------------------------------------------------

export const Paperclip = createIcon(MedusaPaperClip, 'Paperclip', 'paperclip')
export const Paperplane = createIcon(MedusaPaperPlane, 'Paperplane', 'paperplane')
export const CheckCircle = createIcon(MedusaCheckCircle, 'CheckCircle', 'check-circle')
export const ChartFlow = createIcon(MedusaArrowPath, 'ChartFlow', 'chart-flow')

export const Generate: FC<IconProps> = ({ className, width = 24, height = 24, 'aria-label': ariaLabel, onClick }) => {
  return (
    <MedusaSparkles
      width={width}
      height={height}
      className={`text-[#76b900] ${className ?? ''}`}
      aria-hidden={!ariaLabel}
      aria-label={ariaLabel}
      data-src={`${CDN_BASE}/line/generate.svg`}
      onClick={onClick}
    />
  )
}

export const StopCircle = createIcon(MedusaStopCircleSolid, 'StopCircle', 'shape-circle-off')
export const ThinkingReasoning = createIcon(MedusaAiAssistent, 'ThinkingReasoning', 'neural-network')

// ---------------------------------------------------------------------------
// Loading spinner
// ---------------------------------------------------------------------------

interface LoadingSpinnerProps {
  className?: string
  size?: 'small' | 'medium'
  'aria-label'?: string
}

export const LoadingSpinner: FC<LoadingSpinnerProps> = ({
  className,
  size = 'small',
  'aria-label': ariaLabel = 'Loading',
}) => {
  const sizeClass = size === 'medium' ? 'w-6 h-6' : 'w-4 h-4'
  return (
    <MedusaSpinner
      className={`animate-spin ${sizeClass} text-ui-fg-interactive ${className ?? ''}`}
      aria-label={ariaLabel}
      data-src={`${CDN_BASE}/line/circle-3-q.svg`}
      role="status"
    />
  )
}
