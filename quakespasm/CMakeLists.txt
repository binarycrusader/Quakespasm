cmake_minimum_required(VERSION 3.15)
project(quakespasm)

set(CMAKE_CXX_STANDARD 11)

option(USE_CHROMA "rust-based implementation" ON)

# Source lists
set(QS_COM_SRC
        bgmusic.c
        cd_null.c
        cfgfile.c
        chase.c
        cl_demo.c
        cl_input.c
        cl_main.c
        cl_parse.c
        cl_tent.c
        cmd.c
        common.c
        console.c
        crc.c
        cvar.c
        gl_draw.c
        gl_fog.c
        gl_mesh.c
        gl_model.c
        gl_refrag.c
        gl_rlight.c
        gl_rmain.c
        gl_rmisc.c
        gl_screen.c
        gl_sky.c
        gl_texmgr.c
        gl_vidsdl.c
        gl_warp.c
        host.c
        host_cmd.c
        image.c
        in_sdl.c
        keys.c
        main_sdl.c
        mathlib.c
        menu.c
        net_dgrm.c
        net_loop.c
        net_main.c
        pr_cmds.c
        pr_edict.c
        pr_exec.c
        r_alias.c
        r_brush.c
        r_part.c
        r_sprite.c
        r_world.c
        sbar.c
        snd_codec.c
        snd_dma.c
        snd_mem.c
        snd_mix.c
        snd_sdl.c
        snd_vorbis.c
        strlcat.c
        strlcpy.c
        sv_main.c
        sv_move.c
        sv_phys.c
        sv_user.c
        view.c
        wad.c
        world.c
        zone.c)

set(QS_COM_HDR
        anorm_dots.h
        anorms.h
        arch_def.h
        bgmusic.h
        bspfile.h
        cfgfile.h
        client.h
        cmd.h
        common.h
        console.h
        crc.h
        cvar.h
        draw.h
        filenames.h
        gl_model.h
        gl_texmgr.h
        gl_warp_sin.h
        glquake.h
        image.h
        input.h
        keys.h
        lodepng.h
        mathlib.h
        menu.h
        modelgen.h
        net.h
        net_defs.h
        net_dgrm.h
        net_loop.h
        net_sys.h
        platform.h
        pr_comp.h
        progdefs.h
        progdefs.q1
        progs.h
        protocol.h
        q_sound.h
        q_stdinc.h
        qs_bmp.h
        quakedef.h
        render.h
        resource.h
        sbar.h
        screen.h
        server.h
        snd_codec.h
        snd_codeci.h
        spritegn.h
        stb_image_write.h
        strl_fn.h
        sys.h
        vid.h
        view.h
        wad.h
        world.h
        zone.h)

set(QS_CODEC_WAVE_SRC
        snd_wave.c)
set(QS_CODEC_WAVE_HDR
        snd_wave.h)

set(QS_CODEC_FLAC_SRC
        snd_flac.c)
set(QS_CODEC_FLAC_HDR
        snd_flac.h)

set(QS_CODEC_MP3_SRC
        snd_mp3.c)
set(QS_CODEC_MP3_HDR
        snd_mp3.h)

set(QS_CODEC_VORBIS_SRC
        snd_vorbis.c)
set(QS_CODEC_VORBIS_HDR
        snd_vorbis.h)

set(QS_CODEC_OPUS_SRC
        snd_opus.c)
set(QS_CODEC_OPUS_HDR
        snd_opus.h)

set(QS_CODEC_MIKMOD_SRC
        snd_mikmod.c)
set(QS_CODEC_MIKMOD_HDR
        snd_mikmod.h)

set(QS_CODEC_XMP_SRC
        snd_xmp.c)
set(QS_CODEC_XMP_HDR
        snd_xmp.h)

set(QS_CODEC_UMX_SRC
        snd_umx.c)
set(QS_CODEC_UMX_HDR
        snd_umx.h)

set(QS_PL_WIN_SRC
        net_win.c
        net_wins.c
        net_wipx.c
        pl_win.c
        sys_sdl_win.c)

set(QS_PL_WIN_HDR
        net_wins.h
        net_wipx.h
        wsaerror.h)

# Set directory locations (allowing us to move directories easily)
set(QS_SRC_DIR ${CMAKE_SOURCE_DIR}/Quake)

set(QS_PL_SRC
        ${QS_PL_WIN_SRC}
        ${QS_PL_WIN_HDR})

if (${USE_CHROMA})
    # Remove all source files with a rust-based replacement.
    list(REMOVE_ITEM QS_COM_SRC crc.c strlcpy.c strlcat.c)
endif()

set(PL_INC)
set(PL_LNK)
set(PL_LIB)

# find_package(SDL2 REQUIRED)
list(APPEND PL_INC "${CMAKE_SOURCE_DIR}/Windows/SDL2/include")
list(APPEND PL_LNK "${CMAKE_SOURCE_DIR}/Windows/SDL2/lib64")
list(APPEND PL_LIB -lmingw32 -lSDL2main -lSDL2 -mwindows -lm -lopengl32 -lwinmm)
add_definitions(-DUSE_SDL2 -Dmain=SDL_main)

list(APPEND PL_LIB -lm -lopengl32 -lwinmm -lws2_32)
add_definitions(-D_USE_WINSOCK2)

# Enable/disable codecs for streaming music support
option(USE_CODEC_WAVE ".wav music support" ON)
option(USE_CODEC_FLAC "flac music support" ON)
option(USE_CODEC_MP3 "mp3 music support" ON)
option(USE_CODEC_VORBIS "vorbis music support" ON)
option(USE_CODEC_OPUS "opus music support" ON)
# either mikmod or xmp
option(USE_CODEC_MIKMOD "mikmod music support" OFF)
option(USE_CODEC_XMP "xmp music support" OFF)
option(USE_CODEC_UMX "umx music support" OFF)

# quakespasm support options
set(QS_OPT_SRC)
set(QS_OPT_HDR)
set(QS_OPT_INC)
set(QS_OPT_LIB)
set(QS_OPT_LNK)

set(USE_WIN_CODECS)
if (${USE_CODEC_WAVE})
    add_definitions(-DUSE_CODEC_WAVE)
    list(APPEND QS_OPT_SRC ${QS_CODEC_WAVE_SRC})
    list(APPEND QS_OPT_HDR ${QS_CODEC_WAVE_HDR})
endif ()

if (${USE_CODEC_FLAC})
    add_definitions(-DUSE_CODEC_FLAC)
    set(USE_WIN_CODECS 1)
    list(APPEND QS_OPT_LIB -lFLAC)
    list(APPEND QS_OPT_SRC ${QS_CODEC_FLAC_SRC})
    list(APPEND QS_OPT_HDR ${QS_CODEC_FLAC_HDR})
endif ()

if (${USE_CODEC_MP3})
    add_definitions(-DUSE_CODEC_MP3)
    list(APPEND QS_OPT_LIB -lmad)
    list(APPEND QS_OPT_SRC ${QS_CODEC_MP3_SRC})
    list(APPEND QS_OPT_HDR ${QS_CODEC_MP3_HDR})
endif ()

if (${USE_CODEC_VORBIS})
    add_definitions(-DUSE_CODEC_VORBIS)
    set(USE_WIN_CODECS 1)
    list(APPEND QS_OPT_LIB -lvorbisfile -lvorbis -logg)
    list(APPEND QS_OPT_SRC ${QS_CODEC_VORBIS_SRC})
    list(APPEND QS_OPT_HDR ${QS_CODEC_VORBIS_HDR})
endif ()

if (${USE_CODEC_OPUS})
    add_definitions(-DUSE_CODEC_OPUS)
    set(USE_WIN_CODECS 1)
    list(APPEND QS_OPT_LIB -lopusfile -lopus -logg)
    list(APPEND QS_OPT_SRC ${QS_CODEC_OPUS_SRC})
    list(APPEND QS_OPT_HDR ${QS_CODEC_OPUS_HDR})
endif ()

if (${USE_CODEC_MIKMOD})
    add_definitions(-DUSE_CODEC_MIKMOD)
    set(USE_WIN_CODECS 1)
    list(APPEND QS_OPT_LIB -lmikmod)
    list(APPEND QS_OPT_SRC ${QS_CODEC_MIKMOD_SRC})
    list(APPEND QS_OPT_HDR ${QS_CODEC_MIKMOD_HDR})
endif ()

if (${USE_CODEC_XMP})
    add_definitions(-DUSE_CODEC_XMP)
    set(USE_WIN_CODECS 1)
    list(APPEND QS_OPT_LIB -lxmp)
    list(APPEND QS_OPT_SRC ${QS_CODEC_XMP_SRC})
    list(APPEND QS_OPT_HDR ${QS_CODEC_XMP_HDR})
endif ()

if (${USE_CODEC_UMX})
    add_definitions(-DUSE_CODEC_UMX)
    list(APPEND QS_OPT_SRC ${QS_CODEC_UMX_SRC})
    list(APPEND QS_OPT_HDR ${QS_CODEC_UMX_HDR})
endif ()

if (${USE_WIN_CODECS})
    list(APPEND QS_OPT_INC "${CMAKE_SOURCE_DIR}/Windows/codecs/include")
    list(APPEND QS_OPT_LNK "${CMAKE_SOURCE_DIR}/Windows/codecs/x64")
endif ()

if (${CMAKE_BUILD_TYPE} MATCHES "Debug")
    add_definitions(-DDEBUG)
else ()
    add_definitions(-DNDEBUG)
endif ()

# Prepend absolute root to all source paths
list(TRANSFORM QS_COM_SRC PREPEND "${QS_SRC_DIR}/")
list(TRANSFORM QS_COM_HDR PREPEND "${QS_SRC_DIR}/")
list(TRANSFORM QS_PL_SRC PREPEND "${QS_SRC_DIR}/")
list(TRANSFORM QS_PL_HDR PREPEND "${QS_SRC_DIR}/")
list(TRANSFORM QS_OPT_SRC PREPEND "${QS_SRC_DIR}/")
list(TRANSFORM QS_OPT_HDR PREPEND "${QS_SRC_DIR}/")

if (${USE_CHROMA})
    list(APPEND QS_OPT_LIB chroma -luserenv)
    ADD_LIBRARY(chroma STATIC IMPORTED)
    add_definitions(-DUSE_CHROMA)

    if (${CMAKE_BUILD_TYPE} MATCHES "Debug")
        SET_TARGET_PROPERTIES(chroma PROPERTIES IMPORTED_LOCATION "${CMAKE_SOURCE_DIR}/chromatic/target/debug/chroma.lib")
    else ()
        SET_TARGET_PROPERTIES(chroma PROPERTIES IMPORTED_LOCATION "${CMAKE_SOURCE_DIR}/chromatic/target/release/chroma.lib")
    endif ()
endif ()

# Build quakespasm
include_directories(${QS_SRC_DIR} ${QS_OPT_INC} ${PL_INC})
link_directories(${QS_OPT_LNK} ${PL_LNK})
add_executable(quakespasm ${QS_COM_SRC} ${QS_COM_HDR} ${QS_OPT_SRC} ${QS_OPT_HDR} ${QS_PL_SRC})
target_link_libraries(quakespasm ${QS_OPT_LIB} ${PL_LIB})
