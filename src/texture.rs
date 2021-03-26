pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

#[allow(dead_code)]
fn create_default_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> Texture {
    let size = wgpu::Extent3d {
        width: 1,
        height: 1,
        depth: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Default Texture"),
        size,
        mip_level_count: 1,
        sample_count: Texture::MSAA_SAMPLES,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
    });

    let rgba: [u8; 4] = [255, 20, 150, 255];
    queue.write_texture(
        wgpu::TextureCopyView {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        &rgba,
        wgpu::TextureDataLayout {
            offset: 0,
            bytes_per_row: 4 * size.width,
            rows_per_image: size.height,
        },
        size,
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    Texture {
        texture,
        view,
        sampler,
    }
}

impl Texture {
    #[allow(dead_code)]
    pub fn create<P: AsRef<std::path::Path>>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: P,
    ) -> Texture {
        let mut image = match image::open(path.as_ref()) {
            Ok(img) => img,
            Err(err) => {
                let p: &std::path::Path = path.as_ref();
                log::error!("Failed to load texture {:?} {:?}", p, err);
                return create_default_texture(device, queue);
            }
        };

        let format = match image.color() {
            image::ColorType::L8 => wgpu::TextureFormat::R8Unorm,
            image::ColorType::La8 => wgpu::TextureFormat::Rg8Unorm,
            image::ColorType::Rgba8 => wgpu::TextureFormat::Rgba8UnormSrgb,
            image::ColorType::L16 => wgpu::TextureFormat::R16Uint,
            image::ColorType::La16 => wgpu::TextureFormat::Rg16Uint,
            image::ColorType::Rgba16 => wgpu::TextureFormat::Rgba16Uint,
            image::ColorType::Bgra8 => wgpu::TextureFormat::Bgra8UnormSrgb,
            _ => {
                image = image::DynamicImage::ImageRgba8(image.into_rgba8());
                wgpu::TextureFormat::Rgba8UnormSrgb
            }
        };

        use image::GenericImageView;
        let (width, height) = image.dimensions();

        let size = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            // All textures are stored as 3D, we represent our 2D texture
            // by setting depth to 1.
            size,
            mip_level_count: 1, // We'll talk about this a little later
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: format,
            // SAMPLED tells wgpu that we want to use this texture in shaders
            // COPY_DST means that we want to copy data to this texture
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            label: Some("diffuse_texture"),
        });

        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            image.as_bytes(),
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * size.width,
                rows_per_image: size.height,
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Texture {
            texture,
            view,
            sampler,
        }
    }

    pub const DEPTH_FORMAT : wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub const MSAA_SAMPLES : u32 = 4;
    
    pub fn create_depth_texture(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) -> Self {
        let size = wgpu::Extent3d { // 2.
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: Texture::MSAA_SAMPLES,
            dimension: wgpu::TextureDimension::D2,
            format: Texture::DEPTH_FORMAT,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsage::SAMPLED,
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor { // 4.
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual), // 5.
                lod_min_clamp: -100.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );

        Self { texture, view, sampler }
    }
}
