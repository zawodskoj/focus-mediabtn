MEMORY
{
    sharedram (rwx): ORIGIN = 0x20000000, LENGTH = 0x100
    ram       (rwx): ORIGIN = ORIGIN(sharedram) + LENGTH(sharedram), LENGTH = 20K - LENGTH(sharedram)
    bootrom   (rx) : ORIGIN = 0x08000000, LENGTH = 4K
    approm    (rx) : ORIGIN = ORIGIN(bootrom) + LENGTH(bootrom), LENGTH = 64K - LENGTH(bootrom)
}

SECTIONS
{
    .shared_memory (NOLOAD) : {
        KEEP(*(.shared_memory))
    } >sharedram

    .image_hdr : {
        KEEP (*(.image_hdr))
    } > approm
}

__sharedram_start__ = ORIGIN(sharedram);
__bootrom_start__ = ORIGIN(bootrom);
__approm_start__ = ORIGIN(approm);

