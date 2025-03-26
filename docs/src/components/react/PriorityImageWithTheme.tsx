// Next Imports
import Image from "next/image";

// Utils Imports
import clsx from "clsx";

// Interface for Image Props
interface ImageProps {
  alt: string;
  width: number;
  height: number;
}

// Interface for Priority Image Props, extends Image Props
interface PriorityImageProps extends ImageProps {
  src: string;
  className: string;
}

// Interface for Priority Image With Theme Props, extends Image Props
interface PriorityImageWithThemeProps extends ImageProps {
  darkSrc: string;
  lightSrc: string;
  lightClassName: string;
  darkClassName: string;
  commonClassName: string;
}

// Priority Image Component
export function PriorityImage({
  src,
  alt,
  width,
  height,
  className,
}: PriorityImageProps) {
  return (
    <Image
      src={src}
      alt={alt}
      width={width}
      height={height}
      className={className}
      priority
      loading="eager"
      quality={100}
      placeholder="blur"
      blurDataURL="data:image/jpeg;base64,/9j/4AAQSkZJRgABAQAAAQABAAD/4gHYSUNDX1BST0ZJTEUAAQEAAAHIAAAAAAQwAABtbnRyUkdCIFhZWiAH4AABAAEAAAAAAABhY3NwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAA9tYAAQAAAADTLQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAlkZXNjAAAA8AAAACRyWFlaAAABFAAAABRnWFlaAAABKAAAABRiWFlaAAABPAAAABR3dHB0AAABUAAAABRyVFJDAAABZAAAAChnVFJDAAABZAAAAChiVFJDAAABZAAAAChjcHJ0AAABjAAAADxtbHVjAAAAAAAAAAEAAAAMZW5VUwAAAAgAAAAcAHMAUgBHAEJYWVogAAAAAAAAb6IAADj1AAADkFhZWiAAAAAAAABimQAAt4UAABjaWFlaIAAAAAAAACSgAAAPhAAAts9YWVogAAAAAAAA9tYAAQAAAADTLXBhcmEAAAAAAAQAAAACZmYAAPKnAAANWQAAE9AAAApbAAAAAAAAAABtbHVjAAAAAAAAAAEAAAAMZW5VUwAAACAAAAAcAEcAbwBvAGcAbABlACAASQBuAGMALgAgADIAMAAxADb/2wBDABQODxIPDRQSEBIXFRQdHx4eHRoaHSQtJSEkLUEwLi0tLTAtQFBGRkBQRi4tMCY4PTo+OTFFRkpLRk45OUVFRUX/2wBDAR"
    />
  );
}

// Default Export for Priority Image With Theme Component
export default function PriorityImageWithTheme({
  darkSrc,
  lightSrc,
  alt,
  width,
  height,
  lightClassName,
  darkClassName,
  commonClassName,
}: PriorityImageWithThemeProps) {
  return (
    <>
      <Image
        src={lightSrc}
        alt={alt}
        width={width}
        height={height}
        className={clsx(commonClassName, lightClassName, "block dark:hidden")}
      />
      <Image
        src={darkSrc}
        alt={alt}
        width={width}
        height={height}
        className={clsx(commonClassName, darkClassName, "hidden dark:block")}
      />
    </>
  );
}
