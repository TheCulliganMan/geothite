"""Export Blender's authored corner normals, including smooth and sharp splits.

Call after calc_loop_triangles(). Triangle normals discard smooth shading and
custom split normals; vertex normals discard intentional corner splits.
"""
def transformed_corner_normal(mesh, loop_index, normal_matrix):
    normal = normal_matrix @ mesh.corner_normals[loop_index].vector
    if normal.length_squared < 1e-12:
        raise ValueError("zero authored corner normal")
    return normal.normalized()
