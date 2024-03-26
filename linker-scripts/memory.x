INCLUDE "linker-scripts/memory-map.x"

MEMORY
{
  FLASH : ORIGIN = ORIGIN(approm) + 256, LENGTH = LENGTH(approm) - 256
  RAM   : ORIGIN = ORIGIN(ram), LENGTH = LENGTH(ram)
}

SECTIONS
{
    .image_hdr : {
        KEEP (*(.image_hdr))
    } > approm
}

__image_hdr = ORIGIN(approm);
