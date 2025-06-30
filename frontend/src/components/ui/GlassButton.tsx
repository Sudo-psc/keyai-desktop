import React, { ButtonHTMLAttributes, forwardRef } from 'react';
import { LucideIcon } from 'lucide-react';
import '../styles/liquid-glass.css';

interface GlassButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'success';
  size?: 'sm' | 'md' | 'lg';
  icon?: LucideIcon;
  iconPosition?: 'left' | 'right';
  loading?: boolean;
  children: React.ReactNode;
}

const GlassButton = forwardRef<HTMLButtonElement, GlassButtonProps>(({
  variant = 'primary',
  size = 'md',
  icon: Icon,
  iconPosition = 'left',
  loading = false,
  children,
  className = '',
  disabled,
  ...props
}, ref) => {
  const baseClasses = 'glass-button';
  
  const variantClasses = {
    primary: '',
    secondary: 'glass-button-secondary',
    success: 'glass-button-success'
  };
  
  const sizeClasses = {
    sm: 'glass-button-sm',
    md: '',
    lg: 'glass-button-lg'
  };
  
  const classes = [
    baseClasses,
    variantClasses[variant],
    sizeClasses[size],
    loading && 'glass-shimmer',
    className
  ].filter(Boolean).join(' ');
  
  const iconSize = {
    sm: 14,
    md: 16,
    lg: 18
  };
  
  return (
    <button
      ref={ref}
      className={classes}
      disabled={disabled || loading}
      {...props}
    >
      {loading ? (
        <>
          <div className="animate-spin w-4 h-4 border-2 border-white/20 border-t-white rounded-full" />
          <span>Carregando...</span>
        </>
      ) : (
        <>
          {Icon && iconPosition === 'left' && (
            <Icon size={iconSize[size]} />
          )}
          <span>{children}</span>
          {Icon && iconPosition === 'right' && (
            <Icon size={iconSize[size]} />
          )}
        </>
      )}
    </button>
  );
});

GlassButton.displayName = 'GlassButton';

export default GlassButton; 