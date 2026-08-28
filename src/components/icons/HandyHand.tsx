const HandyHand = ({
  width,
  height,
  className,
}: {
  width?: number | string;
  height?: number | string;
  className?: string;
}) => (
  <img
    src="/brand/kotha-symbol.png"
    width={width || 126}
    height={height || 135}
    className={`kotha-mark ${className ?? ""}`}
    alt=""
    aria-hidden="true"
    draggable={false}
  />
);

export default HandyHand;
