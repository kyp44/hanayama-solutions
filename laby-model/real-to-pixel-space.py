# Real distance from inside track of maze to outside track (cm)
real_maze_thickness = 1.5

# Real nub distance (cm)
real_nub_distance = 1

# Real distance from shoe node to rear nub (cm)
real_shoe_to_rear = 2.3

# Maze distance in pixel space (pix)
pix_maze_thickness = 150

# Conversion factor (pix/cm)
a = pix_maze_thickness / real_maze_thickness

print(f"Nub distance: {a*real_nub_distance} pixels")
print(f"Shoe to rear distance: {a*real_shoe_to_rear} pixels")
