import React, { InputHTMLAttributes, forwardRef, useState } from 'react';
import { LucideIcon, Eye, EyeOff } from 'lucide-react';
import '../styles/liquid-glass.css';

interface GlassInputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  icon?: LucideIcon;
  showPasswordToggle?: boolean;
  containerClassName?: string;
}

const GlassInput = forwardRef<HTMLInputElement, GlassInputProps>(({
  label,
  error,
  icon: Icon,
  showPasswordToggle = false,
  containerClassName = '',
  className = '',
  type = 'text',
  ...props
}, ref) => {
  const [showPassword, setShowPassword] = useState(false);
  const [isFocused, setIsFocused] = useState(false);

  const inputType = showPasswordToggle ? (showPassword ? 'text' : 'password') : type;

  const inputClasses = [
    'glass-input',
    error && 'glass-input-error',
    className
  ].filter(Boolean).join(' ');

  const containerClasses = [
    'glass-input-group',
    containerClassName
  ].filter(Boolean).join(' ');

  return (
    <div className={`space-y-2 ${containerClassName}`}>
      {label && (
        <label className="text-caption text-secondary block">
          {label}
        </label>
      )}

      <div className={containerClasses}>
        {Icon && (
          <Icon
            size={16}
            className={`input-icon transition-colors ${
              isFocused ? 'text-accent' : 'text-secondary'
            }`}
          />
        )}

        <input
          ref={ref}
          type={inputType}
          className={inputClasses}
          onFocus={(e) => {
            setIsFocused(true);
            props.onFocus?.(e);
          }}
          onBlur={(e) => {
            setIsFocused(false);
            props.onBlur?.(e);
          }}
          {...props}
        />

        {showPasswordToggle && (
          <button
            type="button"
            className="absolute right-3 top-1/2 -translate-y-1/2 text-secondary hover:text-accent transition-colors"
            onClick={() => setShowPassword(!showPassword)}
            tabIndex={-1}
          >
            {showPassword ? <EyeOff size={16} /> : <Eye size={16} />}
          </button>
        )}
      </div>

      {error && (
        <span className="text-small text-red-400 block">
          {error}
        </span>
      )}
    </div>
  );
});

GlassInput.displayName = 'GlassInput';

export default GlassInput;
