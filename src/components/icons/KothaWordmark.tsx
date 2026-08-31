import React from "react";

const KothaWordmark = ({
  width,
  height,
  className,
}: {
  width?: number;
  height?: number;
  className?: string;
}) => {
  return (
    <img
      src="/brand/kotha-wordmark.png"
      width={width}
      height={height}
      className={`kotha-wordmark ${className ?? ""}`}
      alt="Kotha"
      draggable={false}
    />
  );
};

export default KothaWordmark;
