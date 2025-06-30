import React, { HTMLAttributes } from 'react';
import { LucideIcon } from 'lucide-react';
import '../styles/liquid-glass.css';

interface GlassCardProps extends HTMLAttributes<HTMLDivElement> {
  variant?: 'default' | 'primary' | 'success';
  size?: 'sm' | 'md' | 'lg';
  hoverable?: boolean;
  icon?: LucideIcon;
  title?: string;
  subtitle?: string;
  children?: React.ReactNode;
}

const GlassCard: React.FC<GlassCardProps> = ({
  variant = 'default',
  size = 'md',
  hoverable = true,
  icon: Icon,
  title,
  subtitle,
  children,
  className = '',
  ...props
}) => {
  const baseClasses = 'floating-card';
  
  const variantClasses = {
    default: '',
    primary: 'floating-card-primary',
    success: 'floating-card-success'
  };
  
  const sizeClasses = {
    sm: 'p-3',
    md: 'p-4',
    lg: 'p-6'
  };
  
  const hoverClasses = hoverable ? 'hover-lift cursor-pointer' : '';
  
  const classes = [
    baseClasses,
    variantClasses[variant],
    sizeClasses[size],
    hoverClasses,
    className
  ].filter(Boolean).join(' ');
  
  return (
    <div className={classes} {...props}>
      {(Icon || title || subtitle) && (
        <div className="flex items-start gap-4 mb-4">
          {Icon && (
            <div className={`
              p-2 rounded-lg
              ${variant === 'primary' ? 'bg-electric-blue/20' : ''}
              ${variant === 'success' ? 'bg-mint-green/20' : ''}
              ${variant === 'default' ? 'bg-white/10' : ''}
            `}>
              <Icon 
                size={20} 
                className={`
                  ${variant === 'primary' ? 'text-electric-blue' : ''}
                  ${variant === 'success' ? 'text-mint-green' : ''}
                  ${variant === 'default' ? 'text-white' : ''}
                `}
              />
            </div>
          )}
          
          <div className="flex-1">
            {title && (
              <h3 className="text-heading text-primary font-semibold">
                {title}
              </h3>
            )}
            {subtitle && (
              <p className="text-caption text-secondary mt-1">
                {subtitle}
              </p>
            )}
          </div>
        </div>
      )}
      
      {children && (
        <div className="text-body text-primary">
          {children}
        </div>
      )}
    </div>
  );
};

export default GlassCard; 