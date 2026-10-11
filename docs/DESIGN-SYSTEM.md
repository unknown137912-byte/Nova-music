# NOVA Music - Design System

This document defines the visual design system for NOVA Music, establishing a premium macOS music application aesthetic that balances sophistication with usability.

---

## 1. Design Philosophy

NOVA Music embraces a **premium, atmospheric** design that feels like a professional music production tool while remaining accessible to all users. The interface prioritizes **visual clarity**, **musical focus**, and **prolonged usability**.

**Core Principles**:
- **Elegance over flash**: Clean, purposeful design with subtle polish
- **Music-first**: Interface adapts to and enhances the listening experience
- **Native feel**: macOS-specific interactions and aesthetics
- **Consistency**: Systematic design patterns across all components
- **Accessibility**: WCAG AA compliance with superior color contrast

---

## 2. Color System

### Primary Palette
```css
--primary-50: #F8F9FA
--primary-100: #E9ECEF
--primary-200: #DEE2E6
--primary-300: #ADB5BD
--primary-400: #6C757D
--primary-500: #495057
--primary-600: #343A40
--primary-700: #212529
--primary-800: #1A1D20
--primary-900: #121316
```

**Usage**: Main backgrounds, primary text, subtle borders

### Accent Palette (PURPLE - Restrained but purposeful)
```css
--accent-50: #F3E8FF
--accent-100: #E9D5FF
--accent-200: #D8B4FE
--accent-300: #C084FC
--accent-400: #A855F7
--accent-500: #9333EA
--accent-600: #7C3AED
--accent-700: #6D28D9
--accent-800: #5B21B6
--accent-900: #4C1D95
```

**Usage**: Primary actions, selected items, accent details, adaptive theme colors

### Semantic Colors
```css
--success: #10B981
--warning: #F59E0B
--error: #EF4444
--info: #3B82F6
--neutral: #6B7280
```

### Theme Variables
```css
/* Dark Theme (Default) */
--bg-primary: #0D0D0D
--bg-secondary: #1A1A1A
--bg-tertiary: #262626
--bg-elevated: #333333
--bg-overlay: rgba(0, 0, 0, 0.8)

--text-primary: #FFFFFF
--text-secondary: #B3B3B3
--text-tertiary: #808080
--text-disabled: #4D4D4D
--text-inverse: #0D0D0D

--border-primary: #333333
--border-secondary: #404040
--border-tertiary: #4D4D4D

--shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.5)
--shadow-md: 0 4px 6px rgba(0, 0, 0, 0.7)
--shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.9)
--shadow-xl: 0 20px 25px rgba(0, 0, 0, 0.95)
```

**Adaptive Theme**: Album artwork colors influence accent palette:
- Extract dominant colors from album art
- Map to --accent-300--800 based on brightness
- Maintain minimum contrast ratios
- Smooth transitions between artwork changes

---

## 3. Typography

### Font Stack
```css
--font-display: "SF Pro Display", "Inter", system-ui, -apple-system, sans-serif
--font-body: "SF Pro Text", "Inter", system-ui, -apple-system, sans-serif
--font-mono: "SF Mono", "SFMono-Regular", Monaco, monospace
```

### Scale System (Based on 16px base)
```css
--text-xs: 0.75rem    /* 12px */
--text-sm: 0.875rem   /* 14px */
--text-base: 1rem     /* 16px */
--text-lg: 1.125rem   /* 18px */
--text-xl: 1.25rem    /* 20px */
--text-2xl: 1.5rem    /* 24px */
--text-3xl: 1.875rem  /* 30px */
--text-4xl: 2.25rem   /* 36px */
--text-5xl: 3rem      /* 48px */
```

### Font Weights
```css
--weight-light: 300
--weight-regular: 400
--weight-medium: 500
--weight-semibold: 600
--weight-bold: 700
--weight-heavy: 800
```

### Heading Hierarchy
```css
h1 {
  font-size: var(--text-4xl);
  font-weight: var(--weight-heavy);
  letter-spacing: -0.025em;
  line-height: 1.2;
}

h2 {
  font-size: var(--text-3xl);
  font-weight: var(--weight-bold);
  letter-spacing: -0.025em;
  line-height: 1.3;
}

h3 {
  font-size: var(--text-2xl);
  font-weight: var(--weight-semibold);
  line-height: 1.4;
}

.h4, h4 {
  font-size: var(--text-xl);
  font-weight: var(--weight-semibold);
  line-height: 1.4;
}

.h5, h5 {
  font-size: var(--text-lg);
  font-weight: var(--weight-medium);
  line-height: 1.5;
}

.h6, h6 {
  font-size: var(--text-base);
  font-weight: var(--weight-medium);
  line-height: 1.6;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
```

---

## 4. Spacing & Layout

### Grid System
4px grid for consistency:
```css
--space-0: 0
--space-1: 4px   /* xs */
--space-2: 8px   /* sm */
--space-3: 12px  /* md */
--space-4: 16px  /* lg */
--space-5: 20px  /* xl */
--space-6: 24px  /* 2xl */
--space-7: 28px  /* 3xl */
--space-8: 32px  /* 4xl */
--space-9: 36px  /* 5xl */
--space-10: 40px  /* 6xl */
--space-11: 44px  /* 7xl */
--space-12: 48px  /* 8xl */
```

### Layout Patterns
```css
/* Container widths */
.container {
  width: 100%;
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 var(--space-6);
}

/* Section spacing */
.section {
  padding: var(--space-10) 0;
  margin: var(--space-10) 0;
}

/* Card component */
.card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 12px;
  padding: var(--space-6);
  transition: all 0.2s ease;
}

.card:hover {
  background: var(--bg-tertiary);
  border-color: var(--border-secondary);
  transform: translateY(-1px);
  box-shadow: var(--shadow-md);
}
```

---

## 5. Component System

### Core Components

#### Button
```css
.button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-4);
  border: none;
  border-radius: 8px;
  background: var(--accent-600);
  color: white;
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
  overflow: hidden;
}

.button:hover {
  background: var(--accent-700);
  transform: translateY(-1px);
  box-shadow: var(--shadow-md);
}

.button:active {
  transform: translateY(0);
}

.button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.button-secondary {
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  color: var(--text-primary);
}

.button-secondary:hover {
  background: var(--bg-tertiary);
  border-color: var(--border-secondary);
}

.button-ghost {
  background: transparent;
  color: var(--text-secondary);
}

.button-ghost:hover {
  background: var(--bg-tertiary);
}

.button-icon {
  width: 36px;
  height: 36px;
  padding: 0;
  border-radius: 50%;
}
```

#### Input
```css
.input {
  width: 100%;
  padding: var(--space-3) var(--space-4);
  border: 1px solid var(--border-primary);
  border-radius: 8px;
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: var(--text-base);
  transition: all 0.2s ease;
}

.input:focus {
  outline: none;
  border-color: var(--accent-600);
  box-shadow: 0 0 0 3px rgba(139, 92, 246, 0.1);
}

.input::placeholder {
  color: var(--text-tertiary);
}
```

#### Card
```css
.card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 12px;
  overflow: hidden;
  transition: all 0.2s ease;
}

.card-header {
  padding: var(--space-5);
  border-bottom: 1px solid var(--border-primary);
}

.card-body {
  padding: var(--space-5);
}

.card-footer {
  padding: var(--space-5);
  border-top: 1px solid var(--border-primary);
}

.card-hover {
  cursor: pointer;
}

.card-hover:hover {
  background: var(--bg-tertiary);
  border-color: var(--border-secondary);
  transform: translateY(-2px);
  box-shadow: var(--shadow-lg);
}
```

#### List Item
```css
.list-item {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  padding: var(--space-3) var(--space-5);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.list-item:hover {
  background: var(--bg-tertiary);
}

.list-item.selected {
  background: rgba(139, 92, 246, 0.1);
  border-left: 3px solid var(--accent-600);
}
```

#### Modal
```css
.modal-overlay {
  position: fixed;
  inset: 0;
  background: var(--bg-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  opacity: 0;
  animation: fadeIn 0.2s ease forwards;
}

.modal {
  background: var(--bg-secondary);
  border-radius: 16px;
  box-shadow: var(--shadow-xl);
  width: 90%;
  max-width: 800px;
  max-height: 90vh;
  overflow: hidden;
  transform: scale(0.95);
  animation: scaleIn 0.2s ease forwards;
}

.modal-header {
  padding: var(--space-6);
  border-bottom: 1px solid var(--border-primary);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.modal-body {
  padding: var(--space-6);
  overflow-y: auto;
  max-height: calc(90vh - 140px);
}
```

#### Progress Bar
```css
.progress {
  width: 100%;
  height: 4px;
  background: var(--bg-tertiary);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent-600);
  transition: width 0.3s ease;
}
```

#### Badge
```css
.badge {
  display: inline-flex;
  align-items: center;
  padding: var(--space-1) var(--space-3);
  border-radius: 20px;
  font-size: var(--text-xs);
  font-weight: var(--weight-medium);
  white-space: nowrap;
}

.badge-primary {
  background: rgba(139, 92, 246, 0.1);
  color: var(--accent-400);
}

.badge-success {
  background: rgba(16, 185, 129, 0.1);
  color: var(--success);
}

.badge-error {
  background: rgba(239, 68, 68, 0.1);
  color: var(--error);
}
```

---

## 6. Iconography

### Icon Set
- **Source**: Hugeicons (free icon set)
- **Style**: Outline, consistent weight, 24px base size
- **Color**: Current color inherits from parent element
- **Size variants**: 16px, 20px, 24px, 32px, 48px

### Icon Usage
```css
.icon {
  width: 24px;
  height: 24px;
  display: inline-block;
  vertical-align: middle;
}

.icon-sm {
  width: 16px;
  height: 16px;
}

.icon-lg {
  width: 32px;
  height: 32px;
}

.icon-xl {
  width: 48px;
  height: 48px;
}
```

### Iconography Principles
- **Functionality over decoration**: Icons serve a clear purpose
- **Consistency**: All icons follow the same visual language
- **Accessibility**: Descriptive labels and screen reader support
- **Size hierarchy**: Icon size indicates importance

---

## 7. Motion & Animation

### Animation Timing Functions
```css
--ease-out-cubic: cubic-bezier(0.215, 0.61, 0.355, 1)
--ease-in-out-cubic: cubic-bezier(0.645, 0.045, 0.355, 1)
--ease-spring: cubic-bezier(0.34, 1.56, 0.64, 1)
```

### Animation Classes
```css
@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes slideUp {
  from { transform: translateY(20px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}

@keyframes scaleIn {
  from { transform: scale(0.95); opacity: 0; }
  to { transform: scale(1); opacity: 1; }
}

.transition-all {
  transition: all 0.2s var(--ease-out-cubic);
}

.transition-transform {
  transition: transform 0.2s var(--ease-out-cubic);
}

.transition-opacity {
  transition: opacity 0.2s var(--ease-out-cubic);
}
```

### Animation Guidelines
- **Entrance**: Fade in + slide up for modal content
- **Exit**: Fade out + scale down for modals
- **Micro-interactions**: Scale on hover for buttons
- **Feedback**: Progress bars for async operations
- **Performance**: Use CSS transforms for animations
- **Reduced motion**: Respect `prefers-reduced-motion`

---

## 8. Visual Effects

### Glassmorphism (Selective)
```css
.glass {
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.glass-dark {
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.05);
}
```

**Usage**: Modal headers, search results, elevated cards

### Shadows
```css
.shadow-xs {
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

.shadow-sm {
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.5);
}

.shadow-md {
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.7);
}

.shadow-lg {
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.9);
}

.shadow-xl {
  box-shadow: 0 16px 32px rgba(0, 0, 0, 1.0);
}
```

### Border Radius
```css
--radius-sm: 4px;
--radius-md: 8px;
--radius-lg: 12px;
--radius-xl: 16px;
--radius-2xl: 24px;
--radius-full: 9999px;
```

---

## 9. Component Library

### Navigation
```css
.nav-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  border-radius: 8px;
  color: var(--text-secondary);
  text-decoration: none;
  transition: all 0.2s ease;
}

.nav-item:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.nav-item.active {
  background: rgba(139, 92, 246, 0.1);
  color: var(--accent-400);
}
```

### Player Controls
```css
.player-control {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.player-control:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.player-control.active {
  color: var(--accent-600);
}

.player-control.play {
  width: 48px;
  height: 48px;
  background: var(--accent-600);
  color: white;
}

.player-control.play:hover {
  background: var(--accent-700);
  transform: scale(1.05);
}
```

### Queue Item
```css
.queue-item {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  padding: var(--space-3);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.queue-item:hover {
  background: var(--bg-tertiary);
}

.queue-item.active {
  background: rgba(139, 92, 246, 0.1);
}

.queue-item.current {
  border-left: 3px solid var(--accent-600);
}

.queue-item-number {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  font-size: var(--text-xs);
  font-weight: var(--weight-medium);
}

.queue-item.current .queue-item-number {
  background: var(--accent-600);
  color: white;
}
```

### Album Art
```css
.album-art {
  width: 100%;
  aspect-ratio: 1;
  border-radius: 8px;
  overflow: hidden;
  position: relative;
}

.album-art img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.album-art-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.2s ease;
}

.album-art:hover .album-art-overlay {
  opacity: 1;
}
```

### Loading States
```css
.loading {
  display: inline-block;
  width: 20px;
  height: 20px;
  border: 2px solid var(--border-primary);
  border-radius: 50%;
  border-top-color: var(--accent-600);
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.skeleton {
  background: linear-gradient(90deg,
    var(--bg-secondary) 25%,
    var(--bg-tertiary) 50%,
    var(--bg-secondary) 75%
  );
  background-size: 200% 100%;
  animation: loading 1.5s infinite;
  border-radius: 4px;
}
```

---

## 10. Accessibility

### Color Contrast
- **Minimum**: 4.5:1 for normal text, 3:1 for large text
- **Focus indicators**: Visible outline with 2px width
- **Keyboard navigation**: Full keyboard accessibility
- **Screen readers**: Semantic HTML, aria labels

### Interactive States
```css
/* Focus visible styles */
.focus-visible {
  outline: 2px solid var(--accent-600);
  outline-offset: 2px;
}

/* High contrast mode support */
@media (prefers-contrast: high) {
  .button {
    border-width: 2px;
    border-color: currentColor;
  }
}

/* Reduced motion support */
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }
}
```

---

## 11. Component Testing

### Visual Tests
- **Snapshot testing**: Component rendering across themes
- **Playwright**: End-to-end user workflows
- **axe-core**: Accessibility compliance
- **Chromatic**: Design consistency across components

### Unit Tests
```javascript
// Example test structure
import { render } from '@testing-library/svelte'
import { describe, it, expect } from 'vitest'
import { Button } from './Button.svelte'

describe('Button', () => {
  it('renders with default props', () => {
    const { container } = render(Button, { primary: false })
    const button = container.querySelector('button')
    expect(button).toBeTruthy()
    expect(button.className).toContain('button-secondary')
  })

  it('applies primary styles when primary=true', () => {
    const { container } = render(Button, { primary: true })
    const button = container.querySelector('button')
    expect(button.className).toContain('button-primary')
  })
})```

---

## 12. Design Tokens JSON

```json
{
  "colors": {
    "primary": {
      "50": "#F8F9FA",
      "900": "#121316"
    },
    "accent": {
      "400": "#A855F7",
      "700": "#6D28D9"
    }
  },
  "spacing": {
    "1": "4px",
    "4": "16px",
    "6": "24px",
    "8": "32px"
  },
  "typography": {
    "fontFamilies": {
      "display": "SF Pro Display, Inter, system-ui, sans-serif",
      "body": "SF Pro Text, Inter, system-ui, sans-serif"
    },
    "fontSizes": {
      "xs": "12px",
      "sm": "14px",
      "base": "16px",
      "lg": "18px",
      "xl": "20px"
    }
  },
  "borderRadius": {
    "md": "8px",
    "lg": "12px"
  }
}
```

---

## 13. Component Documentation

### How to Use Components

#### Button Component
```svelte
<script>
import { Button } from '$lib/components/ui/Button.svelte'
</script>

<Button primary={true} on:click={() => console.log('clicked')}>
  Click me
</Button>

<Button secondary={true}>Secondary action</Button>
<Button ghost={true}>Ghost action</Button>
```

#### Card Component
```svelte
<script>
import { Card, CardHeader, CardBody, CardFooter } from '$lib/components/ui/Card.svelte'
</script>

<Card>
  <CardHeader>
    <h3>Card Title</h3>
  </CardHeader>
  <CardBody>
    <p>Card content goes here</p>
  </CardBody>
  <CardFooter>
    <Button>Action</Button>
  </CardFooter>
</Card>
```

#### Modal Component
```svelte
<script>
import { Modal } from '$lib/components/ui/Modal.svelte'
import { createEventDispatcher } from 'svelte'

const dispatch = createEventDispatcher()

function closeModal() {
  dispatch('close')
}
</script>

<Modal on:close={closeModal}>
  <div slot="header">
    <h2>Modal Title</h2>
  </div>
  <div slot="body">
    <p>Modal content</p>
  </div>
  <div slot="footer">
    <Button on:click={closeModal}>Close</Button>
  </div>
</Modal>
```

---

## 14. Design Guidelines

### Color Usage
1. **Accent colors** for primary actions and selected items
2. **Success/warning/error** for status indicators
3. **Neutral grays** for backgrounds, text, and borders
4. **High contrast** for readability
5. **Limited color palette** for consistency

### Typography Hierarchy
1. **Display** for page titles, hero sections
2. **Heading** for section titles, card headers
3. **Body** for paragraph text, descriptions
4. **Caption** for metadata, timestamps
5. **Label** for form inputs, navigation

### Spacing Scale
- **xs** (4px): Tight spacing, micro-interactions
- **sm** (8px): Small spacing, card padding
- **md** (12px): Medium spacing, section margins
- **lg** (16px): Large spacing, component padding
- **xl** (20px): Extra spacing, page sections

### Component States
- **Default**: Standard appearance
- **Hover**: Interactive feedback
- **Active/Pressed**: User interaction
- **Disabled**: Unavailable state
- **Selected**: Chosen/active state
- **Loading**: Async operation

---

## 15. Maintenance

### Component Updates
- **Pattern**: Update design tokens, then all components
- **Testing**: Visual regression tests for all updates
- **Documentation**: Update component stories and examples
- **Versioning**: Semantic versioning for design system changes

### Design Debt Management
- **Technical debt**: Document and prioritize for next release
- **Visual debt**: Collect user feedback on usability
- **Component debt**: Refactor when used in multiple places
- **Performance debt**: Profile and optimize before release

---

## 16. Design System Maturity

### Current State
- ✅ Color system established
- ✅ Typography scale defined
- ✅ Core components built
- ✅ Animation system implemented
- ✅ Accessibility compliant
- ⚠️ Glassmorphism implementation
- ⚠️ Component testing suite
- ⚠️ Design tokens exported

### Goals for v1.0
- ✅ All design tokens finalized
- ✅ Complete component library
- ✅ Comprehensive testing
- ✅ Storybook documentation
- ✅ Accessibility validation

---

## 17. References

1. **Apple Human Interface Guidelines**: https://developer.apple.com/design/
2. **Material Design**: https://m3.material.io/
3. **WCAG 2.1 Success Criteria**: https://www.w3.org/WAI/WCAG21/Conformance
4. **SF Pro Fonts**: Apple system font specifications
5. **Hugeicons**: https://hugeicons.com/

---

## 18. Acknowledgements

This design system draws inspiration from:
- **Apple design language**: Minimalist, elegant aesthetics
- **Material Design**: Component system, motion
- **Design tokens**: Consistent theming
- **Svelte ecosystem**: Component architecture
- **macOS native apps**: Human interface principles

The design system is continuously evolving with user feedback and usability testing.