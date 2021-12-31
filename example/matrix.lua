Matrix = {
	width = 0,
	height = 0,
	segments = 31
}

function Matrix:new(gpu, c)
	local o = {
		gpu = gpu,
		color = c or {0, 1, 0, 1},
		data = {}
	}
	o.width, o.height = gpu:getSize()
	setmetatable(o, self)
	self.__index = self
	return o
end

function Matrix:tick()
	for i = (#self.data)-1, 1,-1 do
		s = self.data[i]
		if (s.y > self.height+self.segments) then
			table.remove(self.data, i)
		else
			s.y = s.y + 1
		end
	end
	local slice = self:createSlice(self.segments)
	table.insert(self.data, slice)
end

function Matrix:createSlice(n)
	local slice = {symbols = {}}
	slice.x = math.random(0, self.width-1)
	slice.y = 0
	for i = 1,n,1 do
		v = math.random()
		table.insert(slice.symbols, {
			char = string.char(math.random(33, 126)),
			col = {
				v * self.color[1],
				v * self.color[2],
				v * self.color[3],
				1
			}
		})
	end
	slice.symbols[1].col = {1, 1, 1, 1}
	return slice
end

function Matrix:render()
	for _,slice in pairs(self.data) do
		for dy,e in pairs(slice.symbols) do
			c = e.col
			self.gpu:setForeground(c[1], c[2], c[3], c[4])
			self.gpu:setText(slice.x, slice.y - dy, e.char)
		end
		self.gpu:setForeground(0,0,0,0)
		self.gpu:setText(slice.x, slice.y - self.segments - 1, ' ')
	end
end