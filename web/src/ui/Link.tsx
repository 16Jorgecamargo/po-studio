import type { AnchorHTMLAttributes } from 'react';
import { onLinkClick } from '../router';

interface Props extends AnchorHTMLAttributes<HTMLAnchorElement> {
  to: string;
}

export function Link({ to, onClick, ...rest }: Props) {
  return (
    <a
      href={to}
      onClick={(e) => {
        onClick?.(e);
        onLinkClick(e, to);
      }}
      {...rest}
    />
  );
}
