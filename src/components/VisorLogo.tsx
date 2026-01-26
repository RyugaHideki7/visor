import { clsx } from "clsx";
import Image from "next/image";

export type VisorLogoProps = {
  variant?: "color" | "white";
  size?: number;
  className?: string;
  priority?: boolean;
};

export function VisorLogo({
  variant = "color",
  size = 64,
  className,
  priority,
}: VisorLogoProps) {
  const src = variant === "white" ? "/visor-logo-white.svg" : "/visor-logo-color.svg";
  return (
    <Image
      src={src}
      alt="Visor logo"
      width={size}
      height={size}
      priority={priority}
      className={clsx("inline-block", className)}
    />
  );
}
