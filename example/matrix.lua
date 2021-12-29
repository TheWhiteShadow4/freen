Matrix = {
	gpu = nil,
	data = {},
	width = 0,
	height = 0,
	segments = 31
}

function Matrix:new(gpu)
	o = {
		gpu = gpu
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
	slice = self:createSlice(self.segments)
	table.insert(self.data, slice)
end

function Matrix:createSlice(n)
	slice = {symbols = {}}
	slice.x = math.random(0, self.width-1)
	slice.y = 0
	for i = 1,n,1 do
		table.insert(slice.symbols, {
			char = string.char(math.random(33, 126)),
			col =  {0, math.random(), 0, 1}
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